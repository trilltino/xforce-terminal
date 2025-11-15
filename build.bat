@echo off
REM XForce Terminal - Build Script
REM Compiles the project in release mode and updates the executable

echo ============================================================
echo   XForce Terminal - Build Script
echo ============================================================
echo.

REM Check if cargo is available
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo ERROR: Cargo not found! Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo [1/3] Building backend...
cargo build --release --bin backend
if %ERRORLEVEL% neq 0 (
    echo [!] Backend build failed!
    pause
    exit /b 1
)

echo.
echo [2/3] Building terminal...
cargo build --release --bin terminal
if %ERRORLEVEL% neq 0 (
    echo [!] Terminal build failed!
    pause
    exit /b 1
)

echo.
echo [3/3] Building wallet-web (if available)...
if exist wallet-web\Cargo.toml (
    cargo build --release --bin wallet-server --manifest-path wallet-web/Cargo.toml
    if %ERRORLEVEL% neq 0 (
        echo [!] Wallet-web build failed (non-critical)...
    )
)

if %ERRORLEVEL% neq 0 (
    echo.
    echo ============================================================
    echo   BUILD FAILED!
    echo ============================================================
    echo Please check the error messages above.
    pause
    exit /b 1
)

echo.
echo ============================================================
echo   BUILD SUCCESSFUL!
echo ============================================================
echo.
echo Executables built:
if exist target\release\backend.exe (
    echo   [OK] Backend:     target\release\backend.exe
) else (
    echo   [SKIP] Backend:   Not built
)
if exist target\release\terminal.exe (
    echo   [OK] Terminal:    target\release\terminal.exe
) else (
    echo   [SKIP] Terminal:  Not built
)
if exist target\release\wallet-server.exe (
    echo   [OK] Wallet-Web:  target\release\wallet-server.exe
) else (
    echo   [SKIP] Wallet-Web: Not built
)
echo.
echo To run:
echo   start.bat              (builds and starts all services)
echo   target\release\terminal.exe  (run terminal only)
echo   start-with-debug.bat   (with debugging)
echo.
echo ============================================================
echo.

pause
