@echo off
setlocal EnableExtensions
title Build TF2 STV Parser
set "QUIET="
if /I "%~1"=="--quiet" set "QUIET=1"
set "PARSER_ROOT=%~dp0parser"
if not exist "%PARSER_ROOT%\Cargo.toml" goto :missing
where cargo >nul 2>&1
if errorlevel 1 goto :rust
pushd "%PARSER_ROOT%"
cargo build --release --locked --bin export_all
if errorlevel 1 (popd & goto :fail)
popd
if defined QUIET exit /b 0
echo.
echo PARSER BUILD SUCCEEDED.
echo You can now open TF2_STV_Parser_GUI.exe.
pause
exit /b 0
:missing
echo ERROR: Bundled parser source is missing.
goto :fail
:rust
echo ERROR: cargo was not found. Install Rust from https://rustup.rs/ and reopen this terminal.
:fail
echo.
echo BUILD FAILED. The first error above is the useful one.
if defined QUIET exit /b 1
pause
exit /b 1
