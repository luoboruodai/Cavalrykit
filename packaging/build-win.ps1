$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Install Rust and the Visual Studio C++ build tools before building.'
}
$previous = $env:CARGO_ENCODED_RUSTFLAGS
try {
    # Cargo's encoded flags preserve spaces in repository and profile paths.
    $flags = @('-C', 'target-feature=+crt-static', "--remap-path-prefix=$($PWD.Path)=C:/build/cavalrykit", "--remap-path-prefix=$HOME=C:/build/home")
    $env:CARGO_ENCODED_RUSTFLAGS = $flags -join [char]0x1f
    & cargo build --release --locked --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed: $LASTEXITCODE" }
    & (Join-Path $PSScriptRoot 'package-win.ps1')
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previous
}
