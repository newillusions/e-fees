; E-Fees NSIS installer hooks
; Handles migration from old currentUser installs to perMachine
;
; Background: Before v0.10.22, E-Fees used the default currentUser install mode
; (installs to %LOCALAPPDATA%). In v0.10.22, we switched to perMachine
; (installs to C:\Program Files\). This left old per-user installations
; orphaned with stale desktop shortcuts pointing to the old location.

!macro NSIS_HOOK_PREINSTALL
  ; Check for old per-user installation via HKCU registry
  ; (perMachine installs use HKLM, so this only matches old currentUser installs)
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\com.emittiv.e-fees" "InstallLocation"

  ${If} $0 != ""
    DetailPrint "Found old per-user installation at $0, removing..."

    ; Try to run the old uninstaller silently
    ${If} ${FileExists} "$0\uninstall.exe"
      ExecWait '"$0\uninstall.exe" /S /currentuser' $1

      ${If} $1 == 0
        DetailPrint "Old per-user installation removed successfully"
      ${Else}
        DetailPrint "Old uninstaller exited with code $1, cleaning up manually..."
        RMDir /r "$0"
        DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\com.emittiv.e-fees"
      ${EndIf}
    ${Else}
      ; Uninstaller missing — clean up registry and directory manually
      DetailPrint "Old uninstaller not found, cleaning up manually..."
      RMDir /r "$0"
      DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\com.emittiv.e-fees"
    ${EndIf}

    ; Remove stale per-user desktop and Start Menu shortcuts
    ; (SetShellVarContext current = per-user locations)
    SetShellVarContext current
    Delete "$DESKTOP\E-Fees.lnk"
    Delete "$DESKTOP\e-fees.lnk"
    Delete "$SMPROGRAMS\E-Fees\E-Fees.lnk"
    Delete "$SMPROGRAMS\E-Fees\e-fees.lnk"
    Delete "$SMPROGRAMS\e-fees\e-fees.lnk"
    Delete "$SMPROGRAMS\e-fees\E-Fees.lnk"
    RMDir "$SMPROGRAMS\E-Fees"
    RMDir "$SMPROGRAMS\e-fees"
    ; Restore context for perMachine install
    SetShellVarContext all
  ${EndIf}
!macroend
