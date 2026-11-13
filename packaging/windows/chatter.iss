; Chatter's Windows installer. scripts/package-windows.py compiles it with
; Inno Setup 7 under Wine, from the same files as the portable ZIP.
;
; It installs for the person running it, without administrator rights, and can
; start Chatter in the tray when they sign in. That is the Run key value
; Settings > Advanced > "Start when you sign in" writes (src/autostart.rs), so
; the installer and the setting turn the same thing on and off.

#ifndef AppVersion
  #error Compile through scripts/package-windows.py, which defines AppVersion, VersionNumber and SourceDir
#endif
; The full preview defines Edition and FileSuffix; the Discord release neither.
; Both share the AppId, so installing one replaces the other.
#ifdef Edition
  #define DisplayName "Chatter (" + Edition + ")"
#else
  #define DisplayName "Chatter"
  #define FileSuffix ""
#endif

[Setup]
; Windows knows an installed Chatter by this; never change it.
AppId={{19FF8D53-8D7E-4E66-91BB-E2F5BE307E9A}
AppName=Chatter
AppVersion={#AppVersion}
AppVerName={#DisplayName} {#AppVersion}
AppPublisher=Offset Studio
AppPublisherURL=https://github.com/granndpa/chatter-beta
AppSupportURL=https://github.com/granndpa/chatter-beta/issues
AppUpdatesURL=https://github.com/granndpa/chatter-beta/releases
VersionInfoVersion={#VersionNumber}
; Per user: %LOCALAPPDATA%\Programs\Chatter, with no UAC prompt.
PrivilegesRequired=lowest
DefaultDirName={autopf}\Chatter
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputBaseFilename=Chatter-{#AppVersion}{#FileSuffix}-windows-x64-setup
SetupIconFile=chatter.ico
WizardImageFile=wizard-large-100.bmp,wizard-large-125.bmp,wizard-large-150.bmp,wizard-large-200.bmp
WizardSmallImageFile=wizard-small-100.bmp,wizard-small-125.bmp,wizard-small-150.bmp,wizard-small-200.bmp
WizardImageStretch=no
UninstallDisplayIcon={app}\chatter.exe
UninstallDisplayName={#DisplayName}
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
; Chatter keeps running in the tray with its window closed, so an update or
; uninstall closes it rather than asking.
CloseApplications=force
RestartApplications=no
#ifdef LicenseFile
LicenseFile={#LicenseFile}
#endif

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
english.StartOnSignIn=Start Chatter automatically when Windows starts
english.StartOnSignInGroup=Startup:

[Tasks]
Name: "startonsignin"; Description: "{cm:StartOnSignIn}"; GroupDescription: "{cm:StartOnSignInGroup}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; README.txt explains the ZIP; the installer needs no instructions.
Source: "{#SourceDir}\*"; Excludes: "README.txt"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
; Copied out by hand for "Just trying it out".
Source: "{#SourceDir}\chatter.exe"; Flags: dontcopy

[Icons]
Name: "{autoprograms}\Chatter"; Filename: "{app}\chatter.exe"
Name: "{autodesktop}\Chatter"; Filename: "{app}\chatter.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Yapfast"; ValueData: """{app}\chatter.exe"" --background"; Tasks: startonsignin
; Chosen here, it starts even if Task Manager had turned it off before.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"; ValueType: none; ValueName: "Yapfast"; Flags: deletevalue dontcreatekey; Tasks: startonsignin
; Chatter's own links open here; claiming Discord's stays the person's choice.
Root: HKCU; Subkey: "Software\Classes\yapfast"; ValueType: string; ValueName: ""; ValueData: "URL:Chatter Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\yapfast"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\yapfast\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\chatter.exe"",0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\yapfast\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\chatter.exe"" --open-link ""%1"""; Flags: uninsdeletekey
; Settings keeps the app discord:// links opened before claiming them here;
; uninstalling gives them back (below) and drops what was kept.
Root: HKCU; Subkey: "Software\Chatter"; Flags: uninsdeletekeyifempty dontcreatekey
Root: HKCU; Subkey: "Software\Yapfast\DiscordProtocol"; Flags: uninsdeletekey dontcreatekey
; Uninstalling stops starting at sign-in, however it was turned on.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "Yapfast"; Flags: uninsdeletevalue dontcreatekey
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"; ValueType: none; ValueName: "Yapfast"; Flags: uninsdeletevalue dontcreatekey

[Run]
Filename: "{app}\chatter.exe"; Description: "{cm:LaunchProgram,Chatter}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
; Before the files go: discord:// links open in the app that had them again,
; if Settings claimed them for this Chatter.
Filename: "{app}\chatter.exe"; Parameters: "--release-discord-links"; Flags: runhidden waituntilterminated; RunOnceId: "ReleaseDiscordLinks"

[Code]
var
  IntroPage: TInputOptionWizardPage;
  UpdatePage: TInputOptionWizardPage;
  Trying: Boolean;

procedure InitializeWizard;
begin
  IntroPage := CreateInputOptionPage(wpWelcome,
    'How do you want to use Chatter?', 'You can always install it later.',
    'Pick one, then click Next.', True, False);
  IntroPage.Add('Just trying it out (opens Chatter now, nothing gets installed)');
  IntroPage.Add('I want to install Chatter');
  IntroPage.SelectedValueIndex := 1;

  UpdatePage := CreateInputOptionPage(IntroPage.ID,
    'Updates', 'How should Chatter stay up to date?',
    'You can change this later in Settings.', True, False);
  UpdatePage.Add('Update automatically (recommended)');
  UpdatePage.Add('Update manually (an Update button shows when a new version is out)');
  UpdatePage.SelectedValueIndex := 0;
end;

{ Copies Chatter to a folder of its own and opens it, without installing. }
procedure TryChatter;
var
  Dir: String;
  Code: Integer;
begin
  ExtractTemporaryFile('chatter.exe');
  Dir := ExpandConstant('{localappdata}\Chatter Trial');
  ForceDirectories(Dir);
  FileCopy(ExpandConstant('{tmp}\chatter.exe'), Dir + '\chatter.exe', False);
  ShellExec('', Dir + '\chatter.exe', '', Dir, SW_SHOWNORMAL, ewNoWait, Code);
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if (CurPageID = IntroPage.ID) and (IntroPage.SelectedValueIndex = 0) then
  begin
    TryChatter;
    Trying := True;
    WizardForm.Close;
    Result := False;
  end;
end;

procedure CancelButtonClick(CurPageID: Integer; var Cancel, Confirm: Boolean);
begin
  if Trying then
    Confirm := False;
end;

{ Chatter reads this once on its next start, then deletes it. }
procedure CurStepChanged(CurStep: TSetupStep);
var
  Dir, Choice: String;
begin
  if CurStep = ssPostInstall then
  begin
    Dir := ExpandConstant('{localappdata}\yapfast\Yapfast\data');
    ForceDirectories(Dir);
    if UpdatePage.SelectedValueIndex = 0 then
      Choice := 'automatic'
    else
      Choice := 'manual';
    SaveStringToFile(Dir + '\installer-updates.txt', Choice, False);
  end;
end;
