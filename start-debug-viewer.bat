@echo off
REM Start the web-based debug log viewer
REM This launches a Python HTTP server and opens the viewer in your browser

echo Starting Debug Log Viewer...
echo.

REM Check if Python is available
python --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Python is not installed or not in PATH
    echo Please install Python 3.6+ to use the web viewer
    echo.
    echo Alternative: Use debug-viewer.py directly
    pause
    exit /b 1
)

REM Start the server in the background
start "Debug Log Server" python debug-viewer-server.py

REM Wait a moment for server to start
timeout /t 2 /nobreak >nul

REM Open browser to the viewer
start http://localhost:8080/viewer

echo.
echo Debug viewer opened in your browser
echo Server is running on http://localhost:8080
echo.
echo Press Ctrl+C in the server window to stop
pause

