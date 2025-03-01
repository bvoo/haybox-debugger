@echo off
setlocal EnableDelayedExpansion

rem Use proper path with quotes
set "WDK_DIR=%ProgramFiles(x86)%\Windows Kits\8.0"

call "%ProgramFiles%\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
cd /d "%~dp0"

echo Using WDK path: !WDK_DIR!
echo Using WDF path: !WDF_PATH!

cargo build