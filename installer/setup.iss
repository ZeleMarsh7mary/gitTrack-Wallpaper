; GitWallpaper — per-user installer, no UAC prompt.
; Build: iscc installer\setup.iss
; Requires: a release build in ..\target\release\git-wallpaper.exe

#define AppName        "GitWallpaper"
#define AppVersion     "0.1.0"
#define AppPublisher   "GitWallpaper"
#define AppExe         "git-wallpaper.exe"
#define BuildDir       "..\target\release"

[Setup]
AppId={{7C1F3B0E-9A4E-4C39-9E4F-2A6D8B5A1C10}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={localappdata}\Programs\GitWallpaper
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=
OutputDir=..\dist
OutputBaseFilename=GitWallpaper-{#AppVersion}-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName={#AppName}
UninstallDisplayIcon={app}\{#AppExe}
SetupIconFile=..\assets\icon.ico
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "autostart"; Description: "Start with Windows"; GroupDescription: "Startup:"; Flags: checkedonce

[Files]
Source: "{#BuildDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\{#AppName}\{#AppName} Settings"; Filename: "{app}\{#AppExe}"; Parameters: "--settings"; IconFilename: "{app}\{#AppExe}"
Name: "{userprograms}\{#AppName}\Uninstall {#AppName}"; Filename: "{uninstallexe}"

[Registry]
; Auto-start lives in HKCU only: the installer never needs admin.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; \
    ValueName: "GitWallpaper"; ValueData: """{app}\{#AppExe}"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
; 1. Start the background daemon.
Filename: "{app}\{#AppExe}"; Description: "Run GitWallpaper in the background"; Flags: nowait postinstall skipifsilent
; 2. Greet the user with the settings window so they can paste repo/username + token.
Filename: "{app}\{#AppExe}"; Parameters: "--settings"; Description: "Open Settings (username or repository URL + token)"; \
    Flags: nowait postinstall skipifsilent

[UninstallRun]
; Kill any running copy first, otherwise the exe stays locked.
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#AppExe}"; Flags: runhidden skipifdoesntexist; RunOnceId: "KillDaemon"
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#AppExe}"; Flags: runhidden skipifdoesntexist; RunOnceId: "KillDaemon2"

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\GitWallpaper\current_wallpaper.png"
Type: filesandordirs; Name: "{localappdata}\GitWallpaper\cache.json"
