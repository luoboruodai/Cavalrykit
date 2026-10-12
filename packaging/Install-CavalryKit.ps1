# Per-user, stable-path install/upgrade. No elevation or file-association changes.
$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'CavalryKit.exe'
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw 'CavalryKit.exe is missing beside this installer.' }
if (Get-Process -Name 'CavalryKit' -ErrorAction SilentlyContinue) {
    throw 'Close all CavalryKit windows before installing. No process has been terminated.'
}
$install = Join-Path $env:LOCALAPPDATA 'Programs\CavalryKit'
$destination = Join-Path $install 'CavalryKit.exe'
if ([IO.Path]::GetFullPath($source) -eq [IO.Path]::GetFullPath($destination)) { throw 'Run this installer from the extracted update folder, not the installation directory.' }
New-Item -ItemType Directory -Force $install | Out-Null
$staged = Join-Path $install ('update-' + [guid]::NewGuid().ToString('N') + '.tmp')
$backup = Join-Path $install ('previous-' + [guid]::NewGuid().ToString('N') + '.bak')
Copy-Item -LiteralPath $source -Destination $staged
try {
    if (Test-Path -LiteralPath $destination) {
        [IO.File]::Replace($staged, $destination, $backup)
    } else { [IO.File]::Move($staged, $destination) }
    $shell = New-Object -ComObject WScript.Shell
    foreach ($folder in @([Environment]::GetFolderPath('Desktop'), [Environment]::GetFolderPath('Programs'))) {
        $shortcut = $shell.CreateShortcut((Join-Path $folder 'CavalryKit.lnk'))
        $shortcut.TargetPath = $destination
        $shortcut.WorkingDirectory = $install
        $shortcut.IconLocation = "$destination,0"
        $shortcut.Description = 'CavalryKit Script and Plugin Installer'
        $shortcut.Save()
    }
    if (Test-Path -LiteralPath $backup) {
        $backups = Join-Path $env:LOCALAPPDATA 'CavalryKit\Backups'
        New-Item -ItemType Directory -Force $backups | Out-Null
        $archive = Join-Path $backups ('CavalryKit-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '.zip')
        Compress-Archive -LiteralPath $backup -DestinationPath $archive
        Remove-Item -LiteralPath $backup
    }
    Write-Host "Installed: $destination"
    Write-Host 'Use the CavalryKit desktop/Start menu shortcut. Remove any older portable copies yourself; they are not deleted automatically.'
    Write-Host 'An unsigned-app SmartScreen prompt is possible. This script does not bypass or disable it.'
} catch {
    if (Test-Path -LiteralPath $backup) {
        [IO.File]::Copy($backup, $destination, $true)
        Write-Warning "Previous executable restored; recovery copy kept at $backup"
    }
    throw
} finally {
    if (Test-Path -LiteralPath $staged) { Remove-Item -LiteralPath $staged }
}
