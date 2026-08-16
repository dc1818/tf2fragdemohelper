@echo off
setlocal EnableExtensions
title Build TF2 STV Parser GUI
echo Building parser...
call "%~dp0Build_Parser_Only.bat" --quiet
if errorlevel 1 goto :fail
set "CSC=%WINDIR%\Microsoft.NET\Framework64\v4.0.30319\csc.exe"
if not exist "%CSC%" set "CSC=%WINDIR%\Microsoft.NET\Framework\v4.0.30319\csc.exe"
if not exist "%CSC%" (
  echo ERROR: The Windows .NET Framework C# compiler was not found.
  goto :fail
)
echo.
echo Building parser GUI...
"%CSC%" /nologo /target:winexe /optimize+ /platform:anycpu /out:"%~dp0TF2_STV_Parser_GUI.exe" /reference:System.dll /reference:System.Core.dll /reference:System.Drawing.dll /reference:System.Windows.Forms.dll /reference:System.Web.Extensions.dll "%~dp0gui\Program.cs" "%~dp0gui\BatchSupport.cs"
if errorlevel 1 goto :fail
echo.
echo BUILD PASSED
echo Double-click TF2_STV_Parser_GUI.exe from now on.
pause
exit /b 0
:fail
echo.
echo BUILD FAILED. Read the first error above.
pause
exit /b 1
