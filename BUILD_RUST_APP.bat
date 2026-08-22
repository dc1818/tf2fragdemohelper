@echo off
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust/Cargo was not found. Install it from https://rustup.rs/ and reopen this terminal.
  exit /b 1
)

cargo build --workspace --release
if errorlevel 1 exit /b %errorlevel%

if not exist dist mkdir dist
copy /y target\release\tf2-frag-helper.exe dist\TF2_Frag_Demo_Helper.exe >nul
copy /y target\release\export_all.exe dist\export_all.exe >nul
echo.
echo Built dist\TF2_Frag_Demo_Helper.exe and dist\export_all.exe
