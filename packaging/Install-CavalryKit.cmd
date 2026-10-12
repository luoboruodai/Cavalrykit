@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-CavalryKit.ps1"
if errorlevel 1 (echo Installation did not finish. Close CavalryKit and retry.)
pause
