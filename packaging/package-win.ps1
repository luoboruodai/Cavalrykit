# Keep the UTF-8 BOM: Windows PowerShell 5.1 must read the Chinese guide filename correctly.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
$version = ((Get-Content Cargo.toml | Select-String '^version = "([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value)
New-Item -ItemType Directory -Force 'dist' | Out-Null
$work = Join-Path 'dist' ('.package-win-' + [guid]::NewGuid().ToString('N'))
$bundle = Join-Path $work 'bundle'
New-Item -ItemType Directory -Force $bundle | Out-Null
$zip = Join-Path 'dist' "CavalryKit-$version-win-x64.zip"
$candidate = Join-Path $work 'candidate.zip'
try {
    Copy-Item 'target/x86_64-pc-windows-msvc/release/cavalrykit.exe' (Join-Path $bundle 'CavalryKit.exe')
    $guides = @('INSTALL.zh-CN.txt', 'INSTALL.en.txt', 'RELEASE_NOTES.md')
    $names = @('CavalryKit.exe', 'Install-CavalryKit.ps1', 'Install-CavalryKit.cmd', 'WINDOWS-使用说明.txt', 'THIRD-PARTY-NOTICES.txt') + $guides
    foreach ($name in ($names | Where-Object { $_ -ne 'CavalryKit.exe' })) {
        $source = if ($name -eq 'RELEASE_NOTES.md') { $name } else { Join-Path 'packaging' $name }
        Copy-Item -LiteralPath $source -Destination (Join-Path $bundle $name)
    }
    foreach ($name in $guides) {
        $text = Get-Content -LiteralPath (Join-Path $bundle $name) -Raw -Encoding UTF8
        if (-not (($text -split "`n")[0].Contains("CavalryKit $version")) -or -not $text.Contains("/releases/tag/v$version")) {
            throw "Guide release version mismatch: $name"
        }
    }
    $members = $names | ForEach-Object { Join-Path $bundle $_ }
    Compress-Archive -LiteralPath $members -DestinationPath $candidate
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $archive = [IO.Compression.ZipFile]::OpenRead((Resolve-Path $candidate).Path)
    try {
        $actual = @($archive.Entries | ForEach-Object { $_.FullName })
        if ($actual.Count -ne $names.Count -or @(Compare-Object $names $actual).Count -ne 0) { throw 'ZIP member allowlist mismatch' }
    } finally { $archive.Dispose() }
    $check = Join-Path $work 'check'
    Expand-Archive -LiteralPath $candidate -DestinationPath $check
    foreach ($name in $names) {
        $sourceHash = (Get-FileHash -LiteralPath (Join-Path $bundle $name) -Algorithm SHA256).Hash
        $packedHash = (Get-FileHash -LiteralPath (Join-Path $check $name) -Algorithm SHA256).Hash
        if ($sourceHash -ne $packedHash) { throw "ZIP member changed: $name" }
    }
    # Candidate and destination share a filesystem; do not overwrite a good ZIP before validation.
    if (Test-Path -LiteralPath $zip) {
        [IO.File]::Replace((Resolve-Path $candidate).Path, (Resolve-Path $zip).Path, $null)
    } else { Move-Item -LiteralPath $candidate -Destination $zip }
    Get-FileHash -LiteralPath $zip -Algorithm SHA256
} finally {
    Remove-Item -LiteralPath $work -Recurse -Force
}
