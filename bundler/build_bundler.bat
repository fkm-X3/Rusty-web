@echo off
setlocal

echo [Bundler Setup] Checking for Visual Studio Environment...

rem Check if cl.exe is already available
where cl.exe >nul 2>nul
if %errorlevel% equ 0 (
    echo [Bundler Setup] cl.exe found in path.
    goto :build
)

rem Try to find vswhere
set "vswhere=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%vswhere%" (
    echo [Bundler Setup] vswhere.exe not found. Checking common paths...
    goto :manual_check
)

rem Use vswhere to find the latest VS installation path
for /f "usebackq tokens=*" %%i in (`"%vswhere%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
    set "vs_path=%%i"
)

if "%vs_path%"=="" (
    echo [Bundler Setup] Visual Studio with C++ tools not found via vswhere.
    goto :manual_check
)

echo [Bundler Setup] Found VS at: %vs_path%
call "%vs_path%\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo [Bundler Setup] Failed to setup VS environment.
    goto :fail
)
goto :build

:manual_check
rem Add manual checks for 2022, 2019, 2017 if vswhere fails or is missing
if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
    call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    goto :build
)
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat" (
    call "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
    goto :build
)

echo [Bundler Setup] Could not locate Visual Studio C++ environment.
echo Please run this script from a 'x64 Native Tools Command Prompt for VS'.
goto :fail

:build
echo [Bundler Setup] Compiling bundler...
cd /d "%~dp0"
if not exist "..\bin" mkdir "..\bin"

cl /nologo /O2 /EHsc /std:c++17 bundler.cpp /Fe:..\bundler.exe
if %errorlevel% neq 0 (
    echo [Bundler Setup] Failed to compile bundler.
    goto :fail
)

echo [Bundler Setup] Bundler compiled successfully.
echo.
echo [Bundler] Running Bundler...
cd ..
bundler.exe

goto :end

:fail
echo [Bundler Setup] Setup failed.
exit /b 1

:end
endlocal
