@echo off
REM Simple batch file to start the web end
cd /d "%~dp0"

echo ============================================
echo  Starting Wallet-Web
echo ============================================
echo.

REM Change to wallet-web directory
cd wallet-web
if %ERRORLEVEL% NEQ 0 (
    echo [!] ERROR: wallet-web directory not found
    pause
    exit /b 1
)

REM Check if dist folder exists and has files
if exist "dist\index.html" (
    echo [*] Found built files in dist/
    echo [*] Starting Python HTTP server...
    echo [*] Server will run at http://localhost:8080/
    echo [*] Press Ctrl+C to stop
    echo.
    
    REM Check if Python is available
    where python >nul 2>nul
    if %ERRORLEVEL% NEQ 0 (
        echo [!] WARNING: Python not found in PATH
        echo [!] Falling back to Trunk watch mode...
        echo.
        goto :start_trunk
    )
    
    python serve.py
    goto :end
)

:start_trunk
REM Check if trunk is installed
where trunk >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [!] ERROR: Trunk is not installed
    echo [!] Install with: cargo install trunk
    pause
    exit /b 1
)

REM Start Trunk in watch mode
if not exist "dist\index.html" (
    echo [*] No built files found in dist/
    echo [*] Starting Trunk in watch mode (development)...
    echo.
)

echo [*] Trunk will watch for changes and rebuild automatically
echo [*] Server will run at http://127.0.0.1:8080/
echo [*] Press Ctrl+C to stop
echo.
trunk serve --port 8080 --address 127.0.0.1

:end
