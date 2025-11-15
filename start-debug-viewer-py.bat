@echo off
REM Start the Python-based debug log viewer (terminal-based)

echo Starting Python Debug Log Viewer...
echo.

REM Check if Python is available
python --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Python is not installed or not in PATH
    echo Please install Python 3.6+ to use this viewer
    pause
    exit /b 1
)

REM Run the Python viewer
python debug-viewer.py %*

pause

