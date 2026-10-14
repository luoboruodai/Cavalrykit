# Keep the UTF-8 BOM: Windows PowerShell 5.1 must read the Chinese guide filename correctly.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$version = ((Get-Content Cargo.toml | Select-String '^version = "([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value)
$bundle = Join-Path 'dist' "CavalryKit-$version-win-x64"
New-Item -ItemType Directory -Force $bundle | Out-Null
Copy-Item 'target/x86_64-pc-windows-msvc/release/cavalrykit.exe' (Join-Path $bundle 'CavalryKit.exe') -Force
Copy-Item 'packaging/Install-CavalryKit.ps1', 'packaging/Install-CavalryKit.cmd', 'packaging/WINDOWS-使用说明.txt', 'packaging/THIRD-PARTY-NOTICES.txt' $bundle -Force
$zip = "$bundle.zip"
# Archive only this reviewed allowlist, never everything left in the staging directory.
$members = @('CavalryKit.exe', 'Install-CavalryKit.ps1', 'Install-CavalryKit.cmd', 'WINDOWS-使用说明.txt', 'THIRD-PARTY-NOTICES.txt') | ForEach-Object { Join-Path $bundle $_ }
Compress-Archive -LiteralPath $members -DestinationPath $zip -Force
Get-FileHash $zip -Algorithm SHA256
