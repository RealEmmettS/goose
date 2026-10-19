{ Shared startup task policy. StartupHive is supplied by each installer edition. }
const
  StartupRunKey = 'Software\Microsoft\Windows\CurrentVersion\Run';
type
  TStartupBytes = array of Byte;
var
  StartupSelectionPrepared: Boolean;
  StartupSelectionError: String;

{ Inno Setup 6 uses a 32-bit setup host even for x64/ARM64 payloads. }
function OpenStartupKey(Key: Longint; SubKey: String; Options, Access: LongWord; var Opened: Longint): Longint;
  external 'RegOpenKeyExW@advapi32.dll stdcall';
function QueryStartupValue(Key: Longint; Name: String; Reserved: Longint; var Kind: LongWord; var Data: Byte; var Size: LongWord): Longint;
  external 'RegQueryValueExW@advapi32.dll stdcall';
function CloseStartupKey(Key: Longint): Longint;
  external 'RegCloseKey@advapi32.dll stdcall';

function ReadOwnedStartup(var Enabled: Boolean): String;
var
  Actual, Expected, Legacy: String;
  RootHandle, Opened: Longint;
  Status: Longint;
  Kind, Size, Index, Character: LongWord;
  Bytes: TStartupBytes;
begin
  Result := '';
  Enabled := False;
  { Use predefined HKEY handles; KEY_WOW64_64KEY selects the payload's view. }
  RootHandle := -2147483646; { HKEY_LOCAL_MACHINE }
  if StartupHive = HKEY_CURRENT_USER then RootHandle := -2147483647;
  Status := OpenStartupKey(RootHandle, StartupRunKey, 0, $0001 or $0100, Opened);
  if Status = 2 then Exit; { ERROR_FILE_NOT_FOUND is the only absent state. }
  if Status <> 0 then begin
    Result := 'The existing Goose startup key cannot be read safely: ' + IntToStr(Status);
    Exit;
  end;
  SetArrayLength(Bytes, 65536);
  Size := 65536;
  try
    Status := QueryStartupValue(Opened, 'Honk300', 0, Kind, Bytes[0], Size);
  finally
    CloseStartupKey(Opened);
  end;
  if Status = 2 then Exit;
  if Status <> 0 then begin
    Result := 'The existing Goose startup value cannot be read safely: ' + IntToStr(Status);
    Exit;
  end;
  if (Kind <> 1) or (Size < 2) or (Size > 65536) or (Size mod 2 <> 0) then begin
    Result := 'The existing Goose startup value has an unsupported type or encoding.';
    Exit;
  end;
  if (Bytes[Size - 2] <> 0) or (Bytes[Size - 1] <> 0) then begin
    Result := 'The existing Goose startup value is not terminated.';
    Exit;
  end;
  SetLength(Actual, (Size div 2) - 1);
  for Index := 1 to Length(Actual) do begin
    Character := Bytes[(Index - 1) * 2] + (Bytes[(Index - 1) * 2 + 1] shl 8);
    if Character = 0 then begin
      Result := 'The existing Goose startup value contains an embedded terminator.';
      Exit;
    end;
    Actual[Index] := Chr(Character);
  end;
  Expected := '"' + ExpandConstant('{app}\bin\honk300-app.exe') + '"';
  Legacy := '"' + ExpandConstant('{app}\bin\honk300.exe') + '" start';
  if (CompareText(Actual, Expected) <> 0) and (CompareText(Actual, Legacy) <> 0) then begin
    Result := 'The existing Goose startup value belongs to another command. Installation has not changed it.';
    Exit;
  end;
  Enabled := True;
end;

function ExplicitStartupTasks: Boolean;
var
  Index: Integer;
  Argument, Tasks: String;
begin
  Result := False;
  for Index := 1 to ParamCount do begin
    Argument := Lowercase(ParamStr(Index));
    { /TASKS replaces all selections; omission is an explicit opt-out.
      /MERGETASKS only changes named tasks. Preserve unrelated task choices. }
    if (Pos('/tasks=', Argument) = 1) or (Pos('/loadinf=', Argument) = 1) then begin
      Result := True;
      Exit;
    end;
    if Pos('/mergetasks=', Argument) = 1 then begin
      Tasks := ',' + Copy(Argument, 13, Length(Argument)) + ',';
      if (Pos(',autostart,', Tasks) > 0) or (Pos(',!autostart,', Tasks) > 0) or (Pos(',*autostart,', Tasks) > 0) then begin
        Result := True;
        Exit;
      end;
    end;
  end;
end;

procedure PrepareStartupSelection;
var
  Enabled: Boolean;
begin
  if StartupSelectionPrepared then Exit;
  StartupSelectionError := ReadOwnedStartup(Enabled);
  if (StartupSelectionError = '') and not ExplicitStartupTasks then begin
    if Enabled then WizardSelectTasks('autostart')
    else WizardSelectTasks('!autostart');
  end;
  StartupSelectionPrepared := True;
end;

procedure CurPageChanged(CurPageID: Integer);
begin
  { Previous/command-line tasks have been loaded by the time this page opens. }
  if CurPageID = wpSelectTasks then PrepareStartupSelection;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Enabled: Boolean;
begin
  { Covers silent installations and a skipped task page. Never reseed after the
    user has seen the tasks, including when navigating Back and Next. }
  PrepareStartupSelection;
  Result := StartupSelectionError;
  if Result = '' then Result := ReadOwnedStartup(Enabled);
end;
