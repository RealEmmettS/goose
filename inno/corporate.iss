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
CloseApplications=yes

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"; Flags: unchecked
Name: "autostart"; Description: "Start honk300 when this user logs in"; GroupDescription: "Startup:"; Flags: unchecked

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\bin"; DestName: "honk300.exe"; Flags: ignoreversion
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\bin"; DestName: "honk.exe"; Flags: ignoreversion
Source: "{#SourceBinDir}\{#MyAppExeName}"; DestDir: "{app}\bin"; DestName: "goose.exe"; Flags: ignoreversion
Source: "install-source-exe-corporate.txt"; DestDir: "{app}"; DestName: "install-source.txt"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_ASSETS.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Honk300"; Filename: "{app}\bin\honk300.exe"; Parameters: "start"; WorkingDir: "{app}\bin"
Name: "{userdesktop}\Honk300"; Filename: "{app}\bin\honk300.exe"; Parameters: "start"; WorkingDir: "{app}\bin"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Honk300"; ValueType: string; ValueName: "InstallSource"; ValueData: "exe-corporate"; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Honk300"; Flags: uninsdeletekeyifempty
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Honk300"; ValueData: """{app}\bin\honk300.exe"" start"; Tasks: autostart; Flags: uninsdeletevalue

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

procedure EnvRemovePath(Path: string);
var
  Paths: string;
  P: Integer;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths) then exit;

  P := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';');
  if P = 0 then exit;

  if P = 1 then
    Delete(Paths, 1, Length(Path) + 1)
  else
    Delete(Paths, P - 1, Length(Path) + 1);

  RegWriteExpandStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    EnvAddPath(ExpandConstant('{app}') + '\bin');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    EnvRemovePath(ExpandConstant('{app}') + '\bin');
end;
