!include "MUI2.nsh"

Name "Afrim Desktop Input Method"
!ifdef VERSION
  OutFile "AfrimDesktop-Setup-${VERSION}-x86_64.exe"
!else
  OutFile "AfrimDesktop-Setup-x86_64.exe"
!endif
InstallDir "$PROGRAMFILES64\afrim-desktop"
RequestExecutionLevel admin

; MUI pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath $INSTDIR
  File "afrim_desktop_tsf.dll"

  ; Register COM/TSF via DllRegisterServer
  ExecWait 'regsvr32 /s "$INSTDIR\afrim_desktop_tsf.dll"'

  ; Grant read+execute to AppContainer apps (UWP/modern apps)
  ExecWait 'icacls "$INSTDIR\afrim_desktop_tsf.dll" /grant *S-1-15-2-1:(RX)'

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Add/Remove Programs entry
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop" \
    "DisplayName" "Afrim Desktop Input Method"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop" \
    "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop" \
    "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop" \
    "Publisher" "fodydev"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop" \
    "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop" \
    "NoRepair" 1
SectionEnd

Section "Uninstall"
  ; Unregister COM/TSF via DllUnregisterServer
  ExecWait 'regsvr32 /u /s "$INSTDIR\afrim_desktop_tsf.dll"'

  Delete "$INSTDIR\afrim_desktop_tsf.dll"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  ; Remove Add/Remove Programs entry
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\afrim-desktop"
SectionEnd
