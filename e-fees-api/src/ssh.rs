//! Reusable SSH operations for remote file system access on the Primary server.

use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::warn;

use crate::config::FolderConfig;
use crate::error::ApiError;

/// Shell-quote a string for safe inclusion in a remote command.
///
/// Wraps the string in single quotes and escapes any embedded single quotes
/// using the `'\''` idiom, which is portable across POSIX shells.
pub(crate) fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// SSH operations helper for executing commands and managing files on a remote host.
pub struct SshOps {
    host: String,
    user: String,
    key_path: String,
}

impl SshOps {
    /// Construct from a `FolderConfig`.
    pub fn from_folder_config(cfg: &FolderConfig) -> Self {
        Self {
            host: cfg.ssh_host.clone(),
            user: cfg.ssh_user.clone(),
            key_path: cfg.ssh_key.clone(),
        }
    }

    /// Common SSH arg prefix: identity, strict-host-key, connect timeout.
    fn ssh_args(&self) -> Vec<String> {
        vec![
            "-i".into(),
            self.key_path.clone(),
            "-o".into(),
            "StrictHostKeyChecking=no".into(),
            "-o".into(),
            "ConnectTimeout=10".into(),
        ]
    }

    /// Execute a single-string remote command and return stdout on success.
    ///
    /// Paths inside `remote_cmd` must be shell-quoted by the caller (use `shell_quote`).
    pub async fn exec(&self, remote_cmd: &str) -> Result<String, ApiError> {
        let user_host = format!("{}@{}", self.user, self.host);

        let output = Command::new("ssh")
            .args(self.ssh_args())
            .arg(&user_host)
            .arg(remote_cmd)
            .output()
            .await
            .map_err(|e| {
                warn!("SSH spawn failed: {}", e);
                ApiError::service_unavailable(format!("SSH connection failed: {}", e))
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            warn!(
                "SSH command failed: stderr={}, stdout={}",
                stderr.trim(),
                stdout.trim()
            );
            Err(ApiError::service_unavailable(format!(
                "SSH command failed: {}",
                stderr.trim()
            )))
        }
    }

    /// Write `content` to `remote_path` via `cat` piped over SSH stdin.
    ///
    /// Creates or overwrites the remote file via `cat > 'path'`.
    pub async fn write_file(&self, remote_path: &str, content: &[u8]) -> Result<(), ApiError> {
        let user_host = format!("{}@{}", self.user, self.host);
        let remote_cmd = format!("cat > {}", shell_quote(remote_path));

        let mut child = Command::new("ssh")
            .args(self.ssh_args())
            .arg(&user_host)
            .arg(&remote_cmd)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                warn!("SSH spawn failed for write_file: {}", e);
                ApiError::service_unavailable(format!("SSH connection failed: {}", e))
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(content).await.map_err(|e| {
                ApiError::service_unavailable(format!("Failed to write SSH stdin: {}", e))
            })?;
        }

        let status = child.wait().await.map_err(|e| {
            ApiError::service_unavailable(format!("SSH write_file wait failed: {}", e))
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(ApiError::service_unavailable(format!(
                "Remote write to {} failed",
                remote_path
            )))
        }
    }

    /// Return `true` if `remote_path` exists on the remote host.
    pub async fn path_exists(&self, remote_path: &str) -> Result<bool, ApiError> {
        let remote_cmd = format!("test -e {} && echo yes || echo no", shell_quote(remote_path));
        let output = self.exec(&remote_cmd).await?;
        Ok(output.trim() == "yes")
    }

    /// Copy a remote file from `from` to `to`.
    pub async fn copy_file(&self, from: &str, to: &str) -> Result<(), ApiError> {
        let remote_cmd = format!("cp {} {}", shell_quote(from), shell_quote(to));
        self.exec(&remote_cmd).await?;
        Ok(())
    }

    /// Create `path` and all parents on the remote host (`mkdir -p`).
    pub async fn mkdir_p(&self, path: &str) -> Result<(), ApiError> {
        let remote_cmd = format!("mkdir -p {}", shell_quote(path));
        self.exec(&remote_cmd).await?;
        Ok(())
    }

    /// Rename/move `from` to `to` on the remote host.
    pub async fn rename(&self, from: &str, to: &str) -> Result<(), ApiError> {
        let remote_cmd = format!("mv {} {}", shell_quote(from), shell_quote(to));
        self.exec(&remote_cmd).await?;
        Ok(())
    }

    /// Trigger a Nextcloud rescan of `subpath`.
    ///
    /// Attempts a targeted OCC scan first; falls back to `--shallow` on group folder 1
    /// if that fails (space-in-path quoting issues across SSH+docker layers are known).
    pub async fn nc_rescan(&self, subpath: &str) -> Result<(), ApiError> {
        let targeted_cmd = format!(
            "docker exec nextcloud-e php occ files:scan --path={}",
            shell_quote(subpath)
        );
        if self.exec(&targeted_cmd).await.is_ok() {
            return Ok(());
        }

        warn!(
            "nc_rescan: targeted scan failed for {}, falling back to --shallow on group folder 1",
            subpath
        );
        let fallback_cmd = "docker exec nextcloud-e php occ groupfolders:scan 1 --shallow";
        self.exec(fallback_cmd).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::shell_quote;

    #[test]
    fn test_shell_quote_simple() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    #[test]
    fn test_shell_quote_with_spaces() {
        assert_eq!(shell_quote("01 Projects"), "'01 Projects'");
    }

    #[test]
    fn test_shell_quote_with_single_quotes() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn test_shell_quote_empty() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn test_shell_quote_only_single_quotes() {
        assert_eq!(shell_quote("'''"), "''\\'''\\'''\\'''");
    }

    #[test]
    fn test_shell_quote_path_with_spaces() {
        assert_eq!(
            shell_quote("/mnt/user/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs"),
            "'/mnt/user/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs'"
        );
    }
}
