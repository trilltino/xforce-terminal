@echo off
echo ============================================
echo  Wallet-Web Development Server (Watch Mode)
echo ============================================
echo.
echo Starting Trunk in watch mode...
echo This will:
echo   - Watch for changes in src/, style/, and index.html
echo   - Automatically rebuild when files change
echo   - Serve on http://127.0.0.1:8080
echo.
echo Press Ctrl+C to stop
echo.

REM Check if trunk is installed
where trunk >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [!] ERROR: Trunk is not installed
    echo [!] Install with: cargo install trunk
    pause
    exit /b 1
)

REM Start trunk serve in watch mode
echo [*] Starting Trunk serve (watch mode)...
echo.
trunk serve --port 8080 --address 127.0.0.1

