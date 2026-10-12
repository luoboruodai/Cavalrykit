# Per-user, stable-path install/upgrade. Optional paths allow isolated Windows CI verification.
param(
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'Programs\CavalryKit'),
    [string[]]$ShortcutFolders = @([Environment]::GetFolderPath('Desktop'), [Environment]::GetFolderPath('Programs')),
    [string]$BackupRoot = (Join-Path $env:LOCALAPPDATA 'CavalryKit\Backups')
)
$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'CavalryKit.exe'
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw 'CavalryKit.exe is missing beside this installer.' }
if (Get-Process -Name 'CavalryKit' -ErrorAction SilentlyContinue) {
    throw 'Close all CavalryKit windows before installing. No process has been terminated.'
}
$install = [IO.Path]::GetFullPath($InstallRoot)
$destination = Join-Path $install 'CavalryKit.exe'
if ([IO.Path]::GetFullPath($source) -eq $destination) { throw 'Run this installer from the extracted update folder, not the installation directory.' }
if ($ShortcutFolders.Count -eq 0) { throw 'At least one shortcut folder is required.' }
$shortcutPaths = @()
# Validate every path before replacing the executable or modifying any shortcut.
foreach ($folder in $ShortcutFolders) {
    if (-not (Test-Path -LiteralPath $folder -PathType Container)) { throw "Shortcut directory is unavailable: $folder" }
    $shortcutPaths += Join-Path $folder 'CavalryKit.lnk'
}
New-Item -ItemType Directory -Force $install | Out-Null
New-Item -ItemType Directory -Force $BackupRoot | Out-Null
$staged = Join-Path $install ('update-' + [guid]::NewGuid().ToString('N') + '.tmp')
$backup = Join-Path $install ('previous-' + [guid]::NewGuid().ToString('N') + '.bak')
$hadExisting = Test-Path -LiteralPath $destination
$shortcutBackups = @{}
foreach ($path in $shortcutPaths) {
    $shortcutBackups[$path] = if (Test-Path -LiteralPath $path) { [IO.File]::ReadAllBytes($path) } else { $null }
}
Copy-Item -LiteralPath $source -Destination $staged
$replaced = $false
try {
    if ($hadExisting) { [IO.File]::Replace($staged, $destination, $backup) }
    else { [IO.File]::Move($staged, $destination) }
    $replaced = $true
    $shell = New-Object -ComObject WScript.Shell
    foreach ($path in $shortcutPaths) {
        $shortcut = $shell.CreateShortcut($path)
        $shortcut.TargetPath = $destination
        $shortcut.WorkingDirectory = $install
        $shortcut.IconLocation = "$destination,0"
        $shortcut.Description = 'CavalryKit Script and Plugin Installer'
        $shortcut.Save()
    }
    if (Test-Path -LiteralPath $backup) {
        $archive = Join-Path $BackupRoot ('CavalryKit-' + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8) + '.zip')
        Compress-Archive -LiteralPath $backup -DestinationPath $archive
        Remove-Item -LiteralPath $backup
    }
    Write-Host "Installed: $destination"
    Write-Host 'Use the CavalryKit desktop/Start menu shortcut. Other portable copies are not deleted automatically.'
    Write-Host 'An unsigned-app SmartScreen prompt is possible. This script does not bypass or disable it.'
} catch {
    $installError = $_
    if ($replaced) {
        if ($hadExisting -and (Test-Path -LiteralPath $backup)) { [IO.File]::Copy($backup, $destination, $true) }
        elseif (-not $hadExisting -and (Test-Path -LiteralPath $destination)) { Remove-Item -LiteralPath $destination }
        foreach ($path in $shortcutPaths) {
            try {
                if ($null -ne $shortcutBackups[$path]) { [IO.File]::WriteAllBytes($path, $shortcutBackups[$path]) }
                elseif (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path }
            } catch { Write-Warning "Shortcut rollback needs attention: $path" }
        }
        if (Test-Path -LiteralPath $backup) { Write-Warning "Recovery copy retained at $backup" }
    }
    throw $installError
} finally {
    if (Test-Path -LiteralPath $staged) { Remove-Item -LiteralPath $staged }
}
