#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{9eccb87e-178f-4493-8ff9-490e024bd34a}
AppName=Mensetsu
AppVersion={#AppVersion}
AppPublisher=Sekai Labs
DefaultDirName={autopf}\Mensetsu
DefaultGroupName=Mensetsu
UninstallDisplayIcon={app}\Mensetsu.exe
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin
OutputDir=..\..\release-assets
OutputBaseFilename=mensetsu-windows-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "..\..\release-assets\windows\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Mensetsu"; Filename: "{app}\Mensetsu.exe"
Name: "{autodesktop}\Mensetsu"; Filename: "{app}\Mensetsu.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Mensetsu.exe"; Description: "Launch Mensetsu"; Flags: postinstall nowait skipifsilent
