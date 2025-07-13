@echo off
echo Building Windows Settings TUI...
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust is not installed or not in PATH
    echo Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Clean previous builds
echo Cleaning previous builds...
cargo clean

REM Build in release mode
echo Building release version...
cargo build --release

if %errorlevel% neq 0 (
    echo Build failed!
    exit /b 1
)

echo.
echo Build successful!
echo Executable location: target\release\windows-settings-tui.exe
echo.

REM Optional: Create a distributable folder
if not exist "dist" mkdir dist
copy target\release\windows-settings-tui.exe dist\
echo Copied to dist\windows-settings-tui.exe

echo.
echo You can now run the application with:
echo   target\release\windows-settings-tui.exe
echo or
echo   dist\windows-settings-tui.exe
echo.