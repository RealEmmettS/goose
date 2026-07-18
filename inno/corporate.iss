#ifndef MyAppVersion
  #define MyAppVersion "0.0.0-dev"
#endif
#ifndef TargetTriple
  #define TargetTriple "x86_64-pc-windows-msvc"
#endif
#ifndef SourceBinDir
  #define SourceBinDir "..\target\release"
#endif
#ifndef InnoArchitecturesAllowed
  #define InnoArchitecturesAllowed "x64"
#endif
#ifndef ReleaseTag
  #define ReleaseTag "v" + MyAppVersion
#endif
#ifndef ReleaseCommit
  #define ReleaseCommit "0000000000000000000000000000000000000000"
#endif
#ifndef PayloadSha256
  #define PayloadSha256 "0000000000000000000000000000000000000000000000000000000000000000"
#endif

#define MyAppName "honk300"
#define MyAppFullName "honk300 (Corporate Edition)"
#define MyAppPublisher "Emmett S"
#define MyAppURL "https://github.com/RealEmmettS/goose"
#define MyAppExeName "honk300.exe"

[Setup]
AppId={{A072F01B-0AE8-4ED9-B67F-845ADF7831F9}
AppName={#MyAppFullName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppFullName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={userpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableDirPage=auto
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=
ArchitecturesAllowed={#InnoArchitecturesAllowed}
ArchitecturesInstallIn64BitMode={#InnoArchitecturesAllowed}
OutputBaseFilename=honk300-{#TargetTriple}-corporate-setup
OutputDir=Output
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
UninstallDisplayName={#MyAppFullName}
SetupLogging=yes
CloseApplications=no

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"; Flags: unchecked
Name: "autostart"; Description: "Start honk300 when this user logs in"; GroupDescription: "Startup:"; Flags: unchecked

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\channels\exe-corporate\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "honk300.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\channels\exe-corporate\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "honk.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\channels\exe-corporate\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "goose.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#SourceBinDir}\honk300-app.exe"; DestDir: "{app}\channels\exe-corporate\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "honk300-app.exe"; Flags: ignoreversion uninsneveruninstall
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_ASSETS.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Honk300"; Filename: "{app}\bin\honk300-app.exe"; WorkingDir: "{app}\bin"; Flags: uninsneveruninstall
Name: "{userdesktop}\Honk300"; Filename: "{app}\bin\honk300-app.exe"; WorkingDir: "{app}\bin"; Tasks: desktopicon; Flags: uninsneveruninstall

[Code]
const
  EnvironmentKey = 'Environment';

procedure EnvAddPath(Path: string);
var
  Paths: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths) then
    Paths := '';

  if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

  if Length(Paths) > 0 then
    Paths := Paths + ';' + Path
  else
    Paths := Path;

  RegWriteExpandStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Helper, Params, AutostartValue: string;
  ResultCode: Integer;
begin
  if CurStep = ssPostInstall then begin
    if WizardIsTaskSelected('autostart') then
      AutostartValue := 'true'
    else
      AutostartValue := 'false';
    Helper := ExpandConstant('{app}\channels\exe-corporate\releases\{#MyAppVersion}-{#TargetTriple}\bin\honk300.exe');
    Params := '__windows-slot-activate --root "' + ExpandConstant('{app}') +
      '" --origin "exe-corporate" --version "{#MyAppVersion}" --tag "{#ReleaseTag}"' +
      ' --commit "{#ReleaseCommit}" --target "{#TargetTriple}"' +
      ' --artifact-name "honk300-{#TargetTriple}-corporate-setup.exe" --artifact-path "' +
      ExpandConstant('{srcexe}') + '" --payload-sha256 "{#PayloadSha256}"' +
      ' --autostart "' + AutostartValue + '"';
    if not Exec(Helper, Params, '', SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
      RaiseException('Honk300 slot activation failed with exit code ' + IntToStr(ResultCode));
    EnvAddPath(ExpandConstant('{app}') + '\bin');
    RegWriteStringValue(HKEY_CURRENT_USER, 'Software\Honk300', 'InstallSource', 'exe-corporate');
    if WizardIsTaskSelected('autostart') then
      RegWriteStringValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run',
        'Honk300', '"' + ExpandConstant('{app}\bin\honk300-app.exe') + '"')
    else
      RegDeleteValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run', 'Honk300');
    if not Exec(Helper, '__windows-slot-commit --root "' + ExpandConstant('{app}') + '"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
      RaiseException('Honk300 slot commit failed with exit code ' + IntToStr(ResultCode));
    if FileExists(ExpandConstant('{app}\.owner-cleanup-pending.json')) and (not WizardSilent) then
      MsgBox('The new Honk300 copy is installed, but an older installer owner still needs cleanup. Run "honk300 update" to finish the verified cleanup.', mbInformation, MB_OK);
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Helper: string;
  ResultCode: Integer;
begin
  if CurUninstallStep = usUninstall then begin
    Helper := ExpandConstant('{app}\bin\honk300.exe');
    if FileExists(Helper) then
      if not Exec(Helper, '__windows-slot-uninstall --root "' + ExpandConstant('{app}') + '" --origin "exe-corporate"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
        RaiseException('Honk300 slot uninstall failed with exit code ' + IntToStr(ResultCode));
  end;
end;
