# Native Windows test: isolated directories and real WScript shortcut creation; never starts Cavalry.
$ErrorActionPreference = 'Stop'
$root = Join-Path ([IO.Path]::GetTempPath()) ('CavalryKit install test ' + [guid]::NewGuid().ToString('N'))
$sourceDir = Join-Path $root 'release input'
$install = Join-Path $root 'Programs\CavalryKit'
$desktop = Join-Path $root 'Desktop'
$programs = Join-Path $root 'Start Menu'
$backups = Join-Path $root 'Backups'
New-Item -ItemType Directory -Force $sourceDir,$desktop,$programs | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'Install-CavalryKit.ps1') $sourceDir
$exe = Join-Path $sourceDir 'CavalryKit.exe'
$script = Join-Path $sourceDir 'Install-CavalryKit.ps1'
$destination = Join-Path $install 'CavalryKit.exe'
function Assert-True($condition, $message) { if (-not $condition) { throw $message } }
try {
    [IO.File]::WriteAllText($exe,'synthetic-version-one')
    & $script -InstallRoot $install -ShortcutFolders @($desktop,$programs) -BackupRoot $backups
    Assert-True ((Get-Content -Raw $destination) -eq 'synthetic-version-one') 'Initial install failed'
    $shell = New-Object -ComObject WScript.Shell
    foreach ($folder in @($desktop,$programs)) {
        $link = $shell.CreateShortcut((Join-Path $folder 'CavalryKit.lnk'))
        Assert-True ($link.TargetPath -eq $destination) 'Shortcut target mismatch'
        Assert-True ($link.IconLocation -eq "$destination,0") 'Icon does not point at the embedded EXE resource'
    }
    [IO.File]::WriteAllText($exe,'synthetic-version-two')
    & $script -InstallRoot $install -ShortcutFolders @($desktop,$programs) -BackupRoot $backups
    Assert-True ((Get-Content -Raw $destination) -eq 'synthetic-version-two') 'Upgrade failed'
    Assert-True (@(Get-ChildItem $install -File).Count -eq 1) 'Duplicate executable or leftover staging'
    Assert-True (@(Get-ChildItem $backups -Filter '*.zip').Count -eq 1) 'Old version backup missing'
    $unpacked = Join-Path $root 'recovery'
    Expand-Archive (Get-ChildItem $backups -Filter '*.zip' | Select-Object -First 1).FullName $unpacked
    Assert-True ((Get-Content -Raw (Get-ChildItem $unpacked -File | Select-Object -First 1).FullName) -eq 'synthetic-version-one') 'Backup contents mismatch'
    # Invalid shortcut location must fail before replacing the current app.
    $failed = $false
    try { & $script -InstallRoot $install -ShortcutFolders @((Join-Path $root 'missing-folder')) -BackupRoot $backups } catch { $failed = $true }
    Assert-True $failed 'Invalid shortcut path unexpectedly succeeded'
    Assert-True ((Get-Content -Raw $destination) -eq 'synthetic-version-two') 'Preflight failure replaced the executable'
    # An externally locked executable must remain untouched, without leftover .tmp files.
    $lock = [IO.File]::Open($destination,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::None)
    $failed = $false
    try { & $script -InstallRoot $install -ShortcutFolders @($desktop,$programs) -BackupRoot $backups } catch { $failed = $true } finally { $lock.Dispose() }
    Assert-True $failed 'Locked executable was unexpectedly replaced'
    Assert-True ((Get-Content -Raw $destination) -eq 'synthetic-version-two') 'Locked executable contents changed'
    Assert-True (@(Get-ChildItem $install -Filter '*.tmp').Count -eq 0) 'Failed install left temporary files'
    Write-Host 'PASS: native Windows install, upgrade, shortcut icons, backup, preflight failure and file-lock safety.'
} finally {
    Remove-Item -LiteralPath $root -Recurse -Force
}
