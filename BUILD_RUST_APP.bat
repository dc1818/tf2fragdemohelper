@echo off
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust/Cargo was not found. Install it from https://rustup.rs/ and reopen this terminal.
  pause
  exit /b 1
)

cargo build --workspace --release
if errorlevel 1 (
  echo.
  echo BUILD FAILED. The first compiler error above is the useful one.
  pause
  exit /b 1
)

if not exist dist mkdir dist
copy /y target\release\tf2-frag-helper.exe dist\TF2_Frag_Demo_Helper.exe >nul
copy /y target\release\export_all.exe dist\export_all.exe >nul
if exist dist\recording_resources_archive rmdir /s /q dist\recording_resources_archive
xcopy /e /i /y recording_resources_archive dist\recording_resources_archive >nul
echo.
echo Built dist\TF2_Frag_Demo_Helper.exe, dist\export_all.exe, and recording resources
echo Open dist\TF2_Frag_Demo_Helper.exe to start the program.
pause
