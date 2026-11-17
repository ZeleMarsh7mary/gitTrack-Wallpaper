; GitTrack Wallpaper — per-user installer, no UAC prompt.
; Build: iscc installer\setup.iss
; Requires: a release build of both binaries in ..\target\release
;           git-track-wallpaper.exe (daemon), git-track-wallpaper-settings.exe (UI)

#define AppName        "GitTrack Wallpaper"
#define AppVersion     "0.1.0"
#define AppPublisher   "GitTrack Wallpaper"
#define AppExe         "git-track-wallpaper.exe"
#define SettingsExe    "git-track-wallpaper-settings.exe"
#define BuildDir       "..\target\release"
#define RunKey         "Software\Microsoft\Windows\CurrentVersion\Run"

[Setup]
AppId={{7C1F3B0E-9A4E-4C39-9E4F-2A6D8B5A1C10}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={localappdata}\Programs\GitTrackWallpaper
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=
OutputDir=..\dist
OutputBaseFilename=GitTrack-Wallpaper-{#AppVersion}-setup
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
; WebView2-hosted settings window: separate process, only alive while open.
Source: "{#BuildDir}\{#SettingsExe}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\{#AppName}\{#AppName} Settings"; Filename: "{app}\{#SettingsExe}"; IconFilename: "{app}\{#SettingsExe}"
Name: "{userprograms}\{#AppName}\Uninstall {#AppName}"; Filename: "{uninstallexe}"

[Registry]
; Auto-start lives in HKCU only, and only ever points at the daemon.
Root: HKCU; Subkey: "{#RunKey}"; ValueType: string; ValueName: "GitTrackWallpaper"; \
    ValueData: """{app}\{#AppExe}"""; Flags: uninsdeletevalue; Tasks: autostart
; Drop the Run value written by the pre-rename build so autorun cannot keep
; pointing at a stale executable.
Root: HKCU; Subkey: "{#RunKey}"; ValueName: "GitWallpaper"; ValueType: none; \
    Flags: dontcreatekey uninsdeletevalue

[Run]
; 1. Start the background daemon.
Filename: "{app}\{#AppExe}"; Description: "Run {#AppName} in the background"; Flags: nowait postinstall skipifsilent
; 2. Greet the user with the settings window so they can paste repo/username + token.
Filename: "{app}\{#SettingsExe}"; Description: "Open Settings (username or repository URL + token)"; \
    Flags: nowait postinstall skipifsilent

[UninstallRun]
; Kill any running copy first, otherwise the exe stays locked.
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#SettingsExe}"; Flags: runhidden skipifdoesntexist; RunOnceId: "KillSettings"
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM {#AppExe}"; Flags: runhidden skipifdoesntexist; RunOnceId: "KillDaemon"

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\GitTrackWallpaper\current_wallpaper.png"
Type: filesandordirs; Name: "{localappdata}\GitTrackWallpaper\cache.json"

[Code]
// A build distributed as "GitWallpaper" may still be running from the old
// install directory. It would keep repainting the wallpaper with its own
// config, so stop it before the new files land.
procedure StopLegacyProcesses;
var
  ResultCode: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM git-wallpaper.exe', '',
       SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM git-wallpaper-settings.exe', '',
       SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
    StopLegacyProcesses;
end;
