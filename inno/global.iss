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
#define MyAppPublisher "Emmett S"
#define MyAppURL "https://github.com/RealEmmettS/goose"
#define MyAppExeName "honk300.exe"

[Setup]
AppId={{5A94FBD0-DA02-4F63-9363-7D9CE0E280F5}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={commonpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableDirPage=auto
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=
ArchitecturesAllowed={#InnoArchitecturesAllowed}
ArchitecturesInstallIn64BitMode={#InnoArchitecturesAllowed}
OutputBaseFilename=honk300-{#TargetTriple}-setup
OutputDir=Output
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
UninstallDisplayName={#MyAppName}
SetupLogging=yes
CloseApplications=no

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"; Flags: unchecked
Name: "autostart"; Description: "Start honk300 when this user logs in"; GroupDescription: "Startup:"; Flags: unchecked

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\channels\exe-global\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "honk300.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\channels\exe-global\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "honk.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\channels\exe-global\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "goose.exe"; Flags: ignoreversion uninsneveruninstall
Source: "{#SourceBinDir}\honk300-app.exe"; DestDir: "{app}\channels\exe-global\releases\{#MyAppVersion}-{#TargetTriple}\bin"; DestName: "honk300-app.exe"; Flags: ignoreversion uninsneveruninstall
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_ASSETS.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Honk300"; Filename: "{app}\bin\honk300-app.exe"; WorkingDir: "{app}\bin"; Flags: uninsneveruninstall
Name: "{commondesktop}\Honk300"; Filename: "{app}\bin\honk300-app.exe"; WorkingDir: "{app}\bin"; Tasks: desktopicon; Flags: uninsneveruninstall

[Code]
const
  EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure EnvAddPath(Path: string);
var
  Paths: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
    Paths := '';

  if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

  if Length(Paths) > 0 then
    Paths := Paths + ';' + Path
  else
    Paths := Path;

  RegWriteExpandStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths);
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
    Helper := ExpandConstant('{app}\channels\exe-global\releases\{#MyAppVersion}-{#TargetTriple}\bin\honk300.exe');
    Params := '__windows-slot-activate --root "' + ExpandConstant('{app}') +
      '" --origin "exe-global" --version "{#MyAppVersion}" --tag "{#ReleaseTag}"' +
      ' --commit "{#ReleaseCommit}" --target "{#TargetTriple}"' +
      ' --artifact-name "honk300-{#TargetTriple}-setup.exe" --artifact-path "' +
      ExpandConstant('{srcexe}') + '" --payload-sha256 "{#PayloadSha256}"' +
      ' --autostart "' + AutostartValue + '"';
    if not Exec(Helper, Params, '', SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
      RaiseException('Honk300 slot activation failed with exit code ' + IntToStr(ResultCode));
    EnvAddPath(ExpandConstant('{app}') + '\bin');
    RegWriteStringValue(HKEY_LOCAL_MACHINE, 'Software\Honk300', 'InstallSource', 'exe-global');
    if WizardIsTaskSelected('autostart') then
      RegWriteStringValue(HKEY_LOCAL_MACHINE, 'Software\Microsoft\Windows\CurrentVersion\Run',
        'Honk300', '"' + ExpandConstant('{app}\bin\honk300-app.exe') + '"')
    else
      RegDeleteValue(HKEY_LOCAL_MACHINE, 'Software\Microsoft\Windows\CurrentVersion\Run', 'Honk300');
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
      if not Exec(Helper, '__windows-slot-uninstall --root "' + ExpandConstant('{app}') + '" --origin "exe-global"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) or (ResultCode <> 0) then
        RaiseException('Honk300 slot uninstall failed with exit code ' + IntToStr(ResultCode));
  end;
end;
