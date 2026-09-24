@echo off
rem ============================================
rem  CavalryKit Windows 一键构建脚本
rem  在一台 Windows 电脑上双击（或 cmd 中运行）本文件，
rem  产出 dist\CavalryKit.exe
rem  前置：会自动安装 Rust 工具链（需要联网）
rem ============================================
setlocal
cd /d "%~dp0\.."

where cargo >nul 2>nul
if errorlevel 1 (
    echo ==^> 未检测到 Rust，开始安装工具链...
    where winget >nul 2>nul
    if not errorlevel 1 (
        winget install -e --id Rustlang.Rustup
    ) else (
        echo 请先到 https://rustup.rs 安装 Rust，然后重新运行本脚本。
        pause
        exit /b 1
    )
    rem 重新加载 PATH
    call "%USERPROFILE%\.cargo\env.bat" 2>nul
)

echo ==^> 编译 release（首次较慢，请耐心等待）
cargo build --release
if errorlevel 1 (
    echo 构建失败。
    pause
    exit /b 1
)

if not exist dist mkdir dist
copy /y target\release\cavalrykit.exe dist\CavalryKit.exe >nul
echo.
echo ==^> 完成: %cd%\dist\CavalryKit.exe
pause
