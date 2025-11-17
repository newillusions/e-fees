# Gitea Actions Setup for Unraid

Complete guide for setting up Gitea Actions runners on Unraid.

## Overview

Your Gitea instance is running in Docker on Unraid. To enable Gitea Actions (CI/CD), you need to add a `gitea-act-runner` container that will execute the workflows.

**Current Setup:**
- **Gitea Container**: Official `gitea/gitea` image
- **Data Location**: `/mnt/user/appdata/gitea`
- **Web Access**: `https://git.mms.name` (port 3000 internally)
- **Network**: Bridge mode

**What We Need:**
- **Act Runner Container**: `gitea/act_runner:latest`
- **Registration Token**: `hDLXLBGCQ25cj2wHW5lz6SyOmGUHfHihZSoV4BnE`
- **Repository**: `martin/fee-prop`

---

## Method 1: Unraid Docker Template (Recommended)

### Step 1: Add Container via Unraid UI

1. **Open Unraid WebUI** → Docker tab
2. **Click "Add Container"**
3. **Fill in these details:**

```
Name: gitea-act-runner
Repository: gitea/act_runner:latest
Network Type: bridge (or same as Gitea container)
```

### Step 2: Volume Mappings

Add these volume mappings:

| Container Path | Host Path | Access Mode |
|----------------|-----------|-------------|
| `/data` | `/mnt/user/appdata/gitea-runner` | Read/Write |
| `/var/run/docker.sock` | `/var/run/docker.sock` | Read/Write |

**Why `/var/run/docker.sock`?**
The runner needs access to Docker to execute workflows in containers.

### Step 3: Environment Variables

Add these environment variables:

| Variable | Value |
|----------|-------|
| `GITEA_INSTANCE_URL` | `https://git.mms.name` |
| `GITEA_RUNNER_REGISTRATION_TOKEN` | `hDLXLBGCQ25cj2wHW5lz6SyOmGUHfHihZSoV4BnE` |
| `GITEA_RUNNER_NAME` | `unraid-runner-01` |
| `GITEA_RUNNER_LABELS` | `ubuntu-latest,macos-latest,windows-latest` |

### Step 4: Start Container

1. Click "Apply"
2. Wait for container to start
3. Check logs: Click container → View Logs

**Expected log output:**
```
INFO Runner registered successfully
INFO Listening for jobs...
```

---

## Method 2: Docker Compose (Alternative)

If you prefer docker-compose, create this file on your Unraid server:

**Location**: `/mnt/user/appdata/gitea-runner/docker-compose.yml`

```yaml
version: '3.8'

services:
  runner:
    image: gitea/act_runner:latest
    container_name: gitea-act-runner
    restart: always
    environment:
      - GITEA_INSTANCE_URL=https://git.mms.name
      - GITEA_RUNNER_REGISTRATION_TOKEN=hDLXLBGCQ25cj2wHW5lz6SyOmGUHfHihZSoV4BnE
      - GITEA_RUNNER_NAME=unraid-runner-01
      - GITEA_RUNNER_LABELS=ubuntu-latest:docker://node:20-bullseye,macos-latest:docker://node:20-bullseye,windows-latest:docker://node:20-bullseye
    volumes:
      - /mnt/user/appdata/gitea-runner:/data
      - /var/run/docker.sock:/var/run/docker.sock
    networks:
      - gitea-network

networks:
  gitea-network:
    external: true
```

**Start it:**
```bash
cd /mnt/user/appdata/gitea-runner
docker-compose up -d
```

---

## Method 3: Manual Docker Command

SSH into your Unraid server and run:

```bash
docker run -d \
  --name gitea-act-runner \
  --restart always \
  -e GITEA_INSTANCE_URL=https://git.mms.name \
  -e GITEA_RUNNER_REGISTRATION_TOKEN=hDLXLBGCQ25cj2wHW5lz6SyOmGUHfHihZSoV4BnE \
  -e GITEA_RUNNER_NAME=unraid-runner-01 \
  -v /mnt/user/appdata/gitea-runner:/data \
  -v /var/run/docker.sock:/var/run/docker.sock \
  gitea/act_runner:latest
```

---

## Step 2: Configure Repository Secret

### Via Gitea Web UI:

1. Go to: `https://git.mms.name/martin/fee-prop/settings/secrets`
2. Click "Add Secret"
3. Name: `RELEASE_TOKEN` (**Important:** Cannot start with `GITEA_` - reserved by Gitea)
4. Value: `ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f`
5. Click "Add Secret"

### Via API (Alternative):

**Note:** Secret names starting with `GITEA_` are reserved and will be rejected.

```bash
curl -X PUT \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/secrets/RELEASE_TOKEN" \
  -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  -H "Content-Type: application/json" \
  -d '{"data": "ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f"}'
```

---

## Step 3: Verify Runner Registration

Check if the runner is registered:

```bash
curl -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/runners" | jq '.'
```

**Expected output:**
```json
{
  "runners": [
    {
      "id": 1,
      "name": "unraid-runner-01",
      "status": "idle"
    }
  ],
  "total_count": 1
}
```

---

## Step 4: Test the Workflow

### Option A: Manual Trigger

```bash
curl -X POST \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/workflows/release-build.yml/dispatches" \
  -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  -H "Content-Type: application/json" \
  -d '{"ref": "main", "inputs": {"version": "0.10.0"}}'
```

### Option B: Push a Tag

```bash
git tag -a v0.11.0-test -m "Test release"
git push origin v0.11.0-test
```

### Monitor Progress:

1. **Via Web UI**: `https://git.mms.name/martin/fee-prop/actions`
2. **Via API**:
```bash
curl -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/runs?limit=5" | jq '.'
```

3. **Via Container Logs**:
```bash
docker logs -f gitea-act-runner
```

---

## Troubleshooting

### Runner Not Registering

**Symptom**: Container starts but doesn't register

**Check**:
```bash
docker logs gitea-act-runner
```

**Common Issues:**
- Wrong `GITEA_INSTANCE_URL` (must be accessible from container)
- Invalid registration token (they expire - generate new one)
- Network connectivity (can container reach Gitea?)

**Fix - Network Issue**:
If using bridge mode, the runner might not be able to reach `https://git.mms.name`. Try:
- Use Gitea's internal IP: `http://172.17.0.X:3000` (find with `docker inspect gitea`)
- Or use custom bridge network shared between containers

### No Jobs Executing

**Symptom**: Runner is idle, jobs stuck in queue

**Check**:
1. Is runner registered? (API check above)
2. Are labels matching?
   - Workflow uses: `runs-on: ubuntu-latest`
   - Runner provides: Check labels in registration

**Fix**:
Update runner labels to match workflow requirements.

### Docker Socket Permission Denied

**Symptom**: "Cannot connect to Docker daemon"

**Fix**:
```bash
# On Unraid server
chmod 666 /var/run/docker.sock
```

Or run runner in privileged mode (less secure):
```bash
docker run --privileged ...
```

### Workflow Fails: Missing Dependencies

**Symptom**: Build fails with "command not found"

**Fix**: The workflow runs in fresh containers. Make sure:
1. Workflow installs all dependencies (Node.js, Rust, etc.)
2. Runner labels match available containers
3. Use Docker images with required tools pre-installed

---

## Advanced Configuration

### Multiple Runners

For better performance, add multiple runners:

```bash
# Runner 2
docker run -d \
  --name gitea-act-runner-02 \
  --restart always \
  -e GITEA_INSTANCE_URL=https://git.mms.name \
  -e GITEA_RUNNER_REGISTRATION_TOKEN=<NEW_TOKEN> \
  -e GITEA_RUNNER_NAME=unraid-runner-02 \
  -v /mnt/user/appdata/gitea-runner-02:/data \
  -v /var/run/docker.sock:/var/run/docker.sock \
  gitea/act_runner:latest
```

### Runner with Specific Labels

```bash
docker run -d \
  --name gitea-act-runner-linux \
  -e GITEA_RUNNER_LABELS=ubuntu-latest,ubuntu-22.04,linux \
  ...
```

### Resource Limits

```bash
docker run -d \
  --cpus="2.0" \
  --memory="4g" \
  ...
```

---

## Monitoring

### Check Runner Health

```bash
# Container status
docker ps | grep gitea-act-runner

# Runner API status
curl -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/runners"

# Recent workflow runs
curl -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/runs?status=completed&limit=10"
```

### Log Files

```bash
# Real-time logs
docker logs -f gitea-act-runner

# Last 100 lines
docker logs --tail 100 gitea-act-runner

# Save logs to file
docker logs gitea-act-runner > /mnt/user/appdata/gitea-runner/runner.log
```

---

## Security Considerations

### Docker Socket Access

Giving container access to `/var/run/docker.sock` is powerful. The runner can:
- ✅ Execute workflows in isolated containers
- ❌ Control all containers on the host (security risk)

**Mitigation:**
- Run on dedicated Unraid server for CI/CD
- Use rootless Docker (advanced)
- Regularly update runner image

### Secrets Management

- Never commit `GITEA_TOKEN` to repository
- Rotate tokens regularly
- Use repository-level secrets (not organization-wide)

### Network Isolation

- Consider dedicated VLAN for CI/CD containers
- Firewall rules to limit runner's network access
- Don't expose runner's ports externally

---

## Unraid-Specific Tips

### Auto-Start on Boot

The runner container should auto-start with `--restart always` flag. Verify:
```bash
docker inspect gitea-act-runner | grep RestartPolicy
```

### Backups

Include runner data in Unraid backups:
- **Appdata Backup**: `/mnt/user/appdata/gitea-runner`
- **Config**: Runner registration persists in `/data`

### Resource Monitoring

Monitor runner impact on Unraid:
- **Dashboard**: Check CPU/RAM usage
- **Docker tab**: View container stats
- **System Log**: `/var/log/syslog` for errors

---

## Summary

**Quick Setup Checklist:**
- [ ] Create gitea-act-runner container on Unraid
- [ ] Configure environment variables (URL, token, name)
- [ ] Mount volumes (`/data` and `/var/run/docker.sock`)
- [ ] Add `GITEA_TOKEN` secret to repository
- [ ] Verify runner registration via API
- [ ] Test workflow with manual trigger
- [ ] Monitor first workflow run

**Registration Token (Active):** `ki147yQJ3fTyYtdEcDmdLQXRbiLtybhdMfSBbHnr`
**Gitea Token (Workflow Secret):** `ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f`

**Note:** Registration tokens can be regenerated via API if needed:
```bash
curl -X POST -H "Authorization: token ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" \
  "https://git.mms.name/api/v1/repos/martin/fee-prop/actions/runners/registration-token"
```

---

## Next Steps

Once the runner is working:
1. Tag a new release to test: `git tag v0.11.0 && git push origin v0.11.0`
2. Watch workflow execute: `https://git.mms.name/martin/fee-prop/actions`
3. Download built artifacts from release page

---

**Last Updated**: November 17, 2025
**Gitea Version**: 1.25.1
**Runner Image**: gitea/act_runner:latest
