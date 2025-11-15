@echo off
setlocal enabledelayedexpansion

REM Change to script directory to ensure we're in the right place
cd /d "%~dp0"

echo ============================================
echo  XForce Terminal - Debug Mode Launcher
echo ============================================
echo.

REM Set environment variables for comprehensive debugging
REM Includes logging for terminal, backend, websocket, and chart operations
set RUST_LOG=terminal=debug,backend=debug,info
set TERMINAL_DEBUG_REALTIME=1
set TERMINAL_TRACE_ENABLED=1
set TERMINAL_FREEZE_THRESHOLD=1000
set TERMINAL_DEBUG_UI=1
set TERMINAL_DEBUG_CHARTS=1
set RUST_BACKTRACE=1

echo [DEBUG] Script started
echo [DEBUG] Current directory: %CD%
echo [DEBUG] Debug environment variables set
echo [DEBUG] RUST_LOG=%RUST_LOG%
echo [DEBUG] TERMINAL_DEBUG_REALTIME=%TERMINAL_DEBUG_REALTIME%
echo [DEBUG] TERMINAL_DEBUG_UI=%TERMINAL_DEBUG_UI%
echo [DEBUG] TERMINAL_DEBUG_CHARTS=%TERMINAL_DEBUG_CHARTS%
echo [DEBUG] RUST_BACKTRACE=%RUST_BACKTRACE%
echo.

echo [*] XForce Terminal - Starting in DEBUG mode...
echo.

REM ============================================
REM OpenSSL Detection (Required for Solana SDK)
REM ============================================
REM Solana SDK requires OpenSSL on Windows via solana-secp256r1-program
REM This cannot be avoided - OpenSSL must be installed

set OPENSSL_FOUND=0
set OPENSSL_PATH=
set OPENSSL_METHOD=

REM Method 1: Check OPENSSL_DIR environment variable
if defined OPENSSL_DIR (
    REM Check standard lib directory
    if exist "!OPENSSL_DIR!\lib\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=!OPENSSL_DIR!"
        set "OPENSSL_METHOD=Environment Variable - OPENSSL_DIR"
    ) else if exist "!OPENSSL_DIR!\lib\ssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=!OPENSSL_DIR!"
        set "OPENSSL_METHOD=Environment Variable - OPENSSL_DIR"
    ) else if exist "!OPENSSL_DIR!\lib\VC\x64\MD\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=!OPENSSL_DIR!"
        set "OPENSSL_METHOD=Environment Variable - OPENSSL_DIR (VC subdirectory)
    ) else if exist "!OPENSSL_DIR!\lib\VC\x64\MDd\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=!OPENSSL_DIR!"
        set "OPENSSL_METHOD=Environment Variable - OPENSSL_DIR (VC subdirectory)
    ) else if exist "!OPENSSL_DIR!\lib\VC\x64\MT\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=!OPENSSL_DIR!"
        set "OPENSSL_METHOD=Environment Variable - OPENSSL_DIR (VC subdirectory)
    ) else if exist "!OPENSSL_DIR!\lib\VC\x64\MTd\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=!OPENSSL_DIR!"
        set "OPENSSL_METHOD=Environment Variable - OPENSSL_DIR (VC subdirectory)
    )
)

REM Method 2: Check vcpkg installation
if %OPENSSL_FOUND% EQU 0 (
    if defined VCPKG_ROOT (
        if exist "!VCPKG_ROOT!\installed\x64-windows\lib\libssl.lib" (
            set OPENSSL_FOUND=1
            set "OPENSSL_PATH=!VCPKG_ROOT!\installed\x64-windows"
            set OPENSSL_METHOD=vcpkg
        ) else if exist "!VCPKG_ROOT!\installed\x64-windows-static\lib\libssl.lib" (
            set OPENSSL_FOUND=1
            set "OPENSSL_PATH=!VCPKG_ROOT!\installed\x64-windows-static"
            set OPENSSL_METHOD=vcpkg - static
        )
    )
)

REM Method 3: Check Chocolatey/Winget installation (common locations)
if %OPENSSL_FOUND% EQU 0 (
    REM Check standard lib directory
    if exist "C:\Program Files\OpenSSL-Win64\lib\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL-Win64"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files"
    ) else if exist "C:\Program Files\OpenSSL\lib\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files"
    ) else if exist "C:\Program Files\OpenSSL-Win64\lib\VC\x64\MD\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL-Win64"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files (VC subdirectory)
    ) else if exist "C:\Program Files\OpenSSL-Win64\lib\VC\x64\MDd\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL-Win64"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files (VC subdirectory)
    ) else if exist "C:\Program Files\OpenSSL-Win64\lib\VC\x64\MT\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL-Win64"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files (VC subdirectory)
    ) else if exist "C:\Program Files\OpenSSL-Win64\lib\VC\x64\MTd\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL-Win64"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files (VC subdirectory)
    ) else if exist "C:\Program Files\OpenSSL\lib\VC\x64\MD\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files\OpenSSL"
        set "OPENSSL_METHOD=Winget/Chocolatey - Program Files (VC subdirectory)
    )
)

REM Method 4: Check Program Files (x86)
if %OPENSSL_FOUND% EQU 0 (
    REM Check standard lib directory
    if exist "C:\Program Files (x86)\OpenSSL-Win64\lib\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files (x86)\OpenSSL-Win64"
        set "OPENSSL_METHOD=Manual Installation - x86"
    ) else if exist "C:\Program Files (x86)\OpenSSL\lib\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files (x86)\OpenSSL"
        set "OPENSSL_METHOD=Manual Installation - x86"
    ) else if exist "C:\Program Files (x86)\OpenSSL-Win64\lib\VC\x64\MD\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files (x86)\OpenSSL-Win64"
        set "OPENSSL_METHOD=Manual Installation - x86 (VC subdirectory)
    ) else if exist "C:\Program Files (x86)\OpenSSL\lib\VC\x64\MD\libssl.lib" (
        set OPENSSL_FOUND=1
        set "OPENSSL_PATH=C:\Program Files (x86)\OpenSSL"
        set "OPENSSL_METHOD=Manual Installation - x86 (VC subdirectory)
    )
)

REM Method 5: Check for OpenSSL executable in PATH (less reliable but useful)
if %OPENSSL_FOUND% EQU 0 (
    where openssl >nul 2>nul
    if !ERRORLEVEL! EQU 0 (
        REM OpenSSL executable found, try to find library
        REM This is a fallback - may not work for compilation
        set "OPENSSL_METHOD=PATH - executable found, library may be missing"
    )
)

REM Display OpenSSL status and set environment variables
echo [DEBUG] OpenSSL detection complete - FOUND: %OPENSSL_FOUND%
if %OPENSSL_FOUND% EQU 1 (
    echo [DEBUG] OpenSSL method: !OPENSSL_METHOD!
    echo [DEBUG] OpenSSL path: !OPENSSL_PATH!
    echo [*] OpenSSL detected: !OPENSSL_METHOD!
    echo [*] Path: !OPENSSL_PATH!
    REM Set OPENSSL_DIR in current session for build process
    set "OPENSSL_DIR=!OPENSSL_PATH!"
    echo [DEBUG] OPENSSL_DIR set to: !OPENSSL_DIR!
    REM Also try to determine OPENSSL_LIB_DIR if libraries are in VC subdirectory
    if exist "!OPENSSL_PATH!\lib\VC\x64\MD\libssl.lib" (
        set "OPENSSL_LIB_DIR=!OPENSSL_PATH!\lib\VC\x64\MD"
        echo [DEBUG] OPENSSL_LIB_DIR set to: !OPENSSL_LIB_DIR!
    ) else if exist "!OPENSSL_PATH!\lib\VC\x64\MDd\libssl.lib" (
        set "OPENSSL_LIB_DIR=!OPENSSL_PATH!\lib\VC\x64\MDd"
        echo [DEBUG] OPENSSL_LIB_DIR set to: !OPENSSL_LIB_DIR!
    ) else if exist "!OPENSSL_PATH!\lib\VC\x64\MT\libssl.lib" (
        set "OPENSSL_LIB_DIR=!OPENSSL_PATH!\lib\VC\x64\MT"
        echo [DEBUG] OPENSSL_LIB_DIR set to: !OPENSSL_LIB_DIR!
    ) else if exist "!OPENSSL_PATH!\lib\VC\x64\MTd\libssl.lib" (
        set "OPENSSL_LIB_DIR=!OPENSSL_PATH!\lib\VC\x64\MTd"
        echo [DEBUG] OPENSSL_LIB_DIR set to: !OPENSSL_LIB_DIR!
    )
    echo [*] OpenSSL is ready - proceeding with build...
    echo.
    goto OPENSSL_DETECTED
) else (
    echo [DEBUG] OpenSSL not found - showing error message
    echo.
    echo ============================================
    echo   OpenSSL Not Found - Installation Required
    echo ============================================
    echo.
    echo [ERROR] OpenSSL is required for Solana SDK dependencies.
    echo [ERROR] The Solana SDK via solana-secp256r1-program requires OpenSSL on Windows.
    echo [ERROR] This is a hard dependency that cannot be avoided.
    echo.
    echo [*] Installation Options:
    echo.
    echo     1. QUICKEST: Run install-openssl.bat (recommended)
    echo        This script will guide you through installation.
    echo.
    echo     2. Chocolatey: choco install openssl
    echo        Requires Chocolatey: https://chocolatey.org/install
    echo.
    echo     3. Manual Download:
    echo        - Download from: https://slproweb.com/products/Win32OpenSSL.html
    echo        - Install to: C:\Program Files\OpenSSL-Win64
    echo        - Set environment variable: setx OPENSSL_DIR "C:\Program Files\OpenSSL-Win64"
    echo.
    echo     4. vcpkg: vcpkg install openssl:x64-windows
    echo        Requires vcpkg: https://github.com/Microsoft/vcpkg
    echo        Set: setx VCPKG_ROOT "C:\path\to\vcpkg"
    echo.
    echo [*] After installation:
    echo     - RESTART YOUR TERMINAL (required for environment variables)
    echo     - Run start-with-debug.bat again
    echo.
    echo [*] Current status: Build will likely fail without OpenSSL
    echo.
    set /p CONTINUE="Continue anyway? (Y/N): "
    if /i "!CONTINUE!" NEQ "Y" (
        echo.
        echo [*] Exiting. Please install OpenSSL and try again.
        echo [*] Run install-openssl.bat for installation help.
        exit /b 1
    )
    echo.
    echo [WARNING] Continuing without OpenSSL - build will likely fail...
    echo.
)

:OPENSSL_DETECTED

REM Set environment variables
set BATCH_SWAP_ROUTER_PROGRAM_ID=HS63bw1V1qTM5uWf92q3uaFdqogrc4SN9qUJSR8aqBMx
set SOLANA_NETWORK=devnet

REM Check for skip-build flag
set SKIP_BUILD=0
if "%1"=="--skip-build" set SKIP_BUILD=1
if "%1"=="-s" set SKIP_BUILD=1

REM Check for clean-build flag (forces full rebuild)
set CLEAN_BUILD=0
if "%1"=="--clean" set CLEAN_BUILD=1
if "%1"=="-c" set CLEAN_BUILD=1
if "%2"=="--clean" set CLEAN_BUILD=1
if "%2"=="-c" set CLEAN_BUILD=1

REM Initialize TRUNK_AVAILABLE variable (needed even when skipping build)
set TRUNK_AVAILABLE=0

REM Determine if we need to build
if %SKIP_BUILD% EQU 1 (
    echo [*] Skipping builds (--skip-build flag set)
    echo [*] Using existing binaries...
    echo [*] Note: Code changes will NOT be applied unless you build
    echo.
    REM Still check if trunk is available for watch mode
    where trunk >nul 2>nul
    if !ERRORLEVEL! EQU 0 (
        if exist wallet-web (
            set TRUNK_AVAILABLE=1
        ) else (
            set TRUNK_AVAILABLE=0
        )
    ) else (
        set TRUNK_AVAILABLE=0
    )
    goto START_SERVERS
)

REM Build to apply code changes (incremental - only compiles changed files)
echo [*] ============================================
echo [*] Building Components (Incremental) - DEBUG MODE
echo [*] ============================================
echo [*] Cargo will only compile files that have changed
echo [*] Backend, Terminal, and Wallet-Web will be built
echo [*] Wallet-Web frontend uses watch mode (auto-rebuilds on changes)
echo [*] All debug logging is enabled
echo.

REM Build backend in release mode (incremental - only changed files)
echo [1/4] Building backend (incremental build)...
echo [DEBUG] Backend build starting...
echo [DEBUG] Clean build flag: %CLEAN_BUILD%
if %CLEAN_BUILD% EQU 1 (
    echo [DEBUG] Clean build requested - cleaning backend first...
    echo [*] Clean build requested - cleaning backend first...
    cargo clean --bin backend
    echo [DEBUG] Clean complete - exit code: !ERRORLEVEL!
)
echo [DEBUG] Starting cargo build for backend...
echo [DEBUG] Command: cargo build --release --bin backend
cargo build --release --bin backend
set BACKEND_BUILD_EXIT_CODE=!ERRORLEVEL!
echo [DEBUG] Backend build complete - exit code: !BACKEND_BUILD_EXIT_CODE!
if !BACKEND_BUILD_EXIT_CODE! NEQ 0 (
    echo.
    echo ============================================
    echo   Backend Build Failed
    echo ============================================
    echo.
    if exist target\release\backend.exe (
        echo [*] Using existing backend binary...
        echo [*] Note: Code changes will NOT be applied
        set BACKEND_BUILD_FAILED=0
    ) else (
        echo [ERROR] Backend build failed - no existing binary found!
        echo [DEBUG] Build exit code: !BACKEND_BUILD_EXIT_CODE!
        echo.
        echo [ERROR] Most likely cause: OpenSSL is not installed or not detected.
        echo [ERROR] The Solana SDK requires OpenSSL on Windows.
        echo.
        exit /b 1
    )
) else (
    set BACKEND_BUILD_FAILED=0
    echo [*] Backend build complete
)
echo.

REM Build terminal in release mode (incremental - only changed files)
echo [2/4] Building terminal (incremental build)...
echo [DEBUG] Terminal build starting...
echo [DEBUG] Clean build flag: %CLEAN_BUILD%
if %CLEAN_BUILD% EQU 1 (
    echo [DEBUG] Clean build requested - cleaning terminal first...
    echo [*] Clean build requested - cleaning terminal first...
    cargo clean --bin terminal
    echo [DEBUG] Clean complete - exit code: !ERRORLEVEL!
)
echo [DEBUG] Starting cargo build for terminal...
echo [DEBUG] Command: cargo build --release --bin terminal
cargo build --release --bin terminal
set TERMINAL_BUILD_EXIT_CODE=!ERRORLEVEL!
echo [DEBUG] Terminal build complete - exit code: !TERMINAL_BUILD_EXIT_CODE!
if !TERMINAL_BUILD_EXIT_CODE! NEQ 0 (
    echo.
    echo ============================================
    echo   Terminal Build Failed - OpenSSL Required
    echo ============================================
    echo.
    echo [ERROR] Terminal build failed due to OpenSSL dependency.
    echo [ERROR] The Solana SDK requires OpenSSL on Windows.
    echo.
    set TERMINAL_BUILD_FAILED=1
) else (
    set TERMINAL_BUILD_FAILED=0
    echo [*] Terminal build complete
)
echo.

REM Build wallet-web server in release mode (incremental - only changed files)
echo [3/4] Building wallet-web server (incremental build)...
echo [DEBUG] Checking for wallet-web...
if exist wallet-web\Cargo.toml (
    echo [DEBUG] wallet-web/Cargo.toml found
    if %CLEAN_BUILD% EQU 1 (
        echo [DEBUG] Clean build requested - cleaning wallet-server first...
        echo [*] Clean build requested - cleaning wallet-server first...
        cargo clean --bin wallet-server --manifest-path wallet-web/Cargo.toml
        echo [DEBUG] Clean complete - exit code: !ERRORLEVEL!
    )
    echo [DEBUG] Starting cargo build for wallet-server...
    echo [DEBUG] Command: cargo build --release --bin wallet-server --manifest-path wallet-web/Cargo.toml
    cargo build --release --bin wallet-server --manifest-path wallet-web/Cargo.toml
    set WALLET_BUILD_EXIT_CODE=!ERRORLEVEL!
    echo [DEBUG] Wallet-web build complete - exit code: !WALLET_BUILD_EXIT_CODE!
    if !WALLET_BUILD_EXIT_CODE! NEQ 0 (
        echo [DEBUG] Wallet-web build failed - exit code: !WALLET_BUILD_EXIT_CODE!
        echo [WARNING] Wallet-web server build failed (non-critical)...
    ) else (
        echo [DEBUG] Wallet-web build succeeded
        echo [*] Wallet-web server build complete - changes applied
    )
    echo.
) else (
    echo [DEBUG] wallet-web/Cargo.toml not found
    echo [3/4] Wallet-web not found, skipping...
    echo.
)

REM Check if trunk is installed for wallet-web frontend
echo [4/4] Setting up wallet-web frontend (watch mode)...
echo [DEBUG] Checking for Trunk installation...
where trunk >nul 2>nul
set TRUNK_CHECK_EXIT=!ERRORLEVEL!
echo [DEBUG] Trunk check exit code: !TRUNK_CHECK_EXIT!
if !TRUNK_CHECK_EXIT! NEQ 0 (
    echo [DEBUG] Trunk not found in PATH
    echo [WARNING] Trunk is not installed
    echo [WARNING] Wallet-web frontend will not be built
    echo [WARNING] Install with: cargo install trunk
    echo.
    set TRUNK_AVAILABLE=0
) else (
    echo [DEBUG] Trunk found in PATH
    if exist wallet-web (
        echo [DEBUG] wallet-web directory exists
        echo [*] Trunk is available - will use trunk serve (watch mode) for auto-reload
        echo [*] This will watch for changes and rebuild automatically
        set TRUNK_AVAILABLE=1
    ) else (
        echo [DEBUG] wallet-web directory not found
        echo [*] Wallet-web directory not found, skipping frontend build...
        set TRUNK_AVAILABLE=0
    )
    echo.
)

echo ============================================
echo  Build Complete!
echo ============================================
echo [*] All components built (incremental - only changed files compiled)
echo [*] Backend, Terminal, and Wallet-Web server are ready
echo [*] Wallet-Web frontend will auto-rebuild on file changes (watch mode)
echo [*] DEBUG MODE: All debug logging is enabled
echo.

REM Create logs directory if it doesn't exist
if not exist logs mkdir logs

REM Clean old realtime log for fresh session
if exist logs\debug-realtime.log del logs\debug-realtime.log

:START_SERVERS
REM Kill any existing servers
echo [*] Cleaning up existing servers...
set CLEANUP_COUNT=0

for /f "tokens=5" %%a in ('netstat -ano 2^>nul ^| findstr ":3001"') do (
    taskkill /F /PID %%a >nul 2>&1
    if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1
)
for /f "tokens=5" %%a in ('netstat -ano 2^>nul ^| findstr ":8080"') do (
    taskkill /F /PID %%a >nul 2>&1
    if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1
)
taskkill /IM wallet-server.exe /F >nul 2>&1
if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1
taskkill /IM backend.exe /F >nul 2>&1
if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1

if !CLEANUP_COUNT! GTR 0 (
    echo [*] Cleaned up !CLEANUP_COUNT! existing process(es)
) else (
    echo [*] No existing servers found
)
timeout /t 1 /nobreak >nul 2>&1

echo.
echo [*] Starting servers...
echo.

REM Start backend server
echo [1/3] Starting backend server on http://localhost:3001...
REM Ensure environment variables are set before starting backend
REM Always use cargo run to ensure latest build is used (even if binary exists)
REM This ensures all code changes are applied
REM Set debug environment variables for backend
REM Store script directory for use in temp batch file
set SCRIPT_DIR=%~dp0
echo @echo off > temp_backend.bat
echo set RUST_LOG=%RUST_LOG% >> temp_backend.bat
echo set RUST_BACKTRACE=%RUST_BACKTRACE% >> temp_backend.bat
echo set BATCH_SWAP_ROUTER_PROGRAM_ID=%BATCH_SWAP_ROUTER_PROGRAM_ID% >> temp_backend.bat
echo set SOLANA_NETWORK=%SOLANA_NETWORK% >> temp_backend.bat
echo cd /d "%SCRIPT_DIR%" >> temp_backend.bat
echo cargo run --release --bin backend >> temp_backend.bat
start "BackendServer" /MIN temp_backend.bat
timeout /t 1 /nobreak >nul 2>&1
del temp_backend.bat >nul 2>&1

REM Wait for backend to start
echo [*] Waiting for backend to initialize...
set RETRY=0
:WAIT_BACKEND
timeout /t 1 /nobreak >nul 2>&1
netstat -ano 2>nul | findstr :3001 | findstr LISTENING > nul
if !ERRORLEVEL! EQU 0 (
    REM Verify health endpoint
    where curl.exe >nul 2>nul
    if !ERRORLEVEL! EQU 0 (
        curl.exe -s http://127.0.0.1:3001/health >nul 2>&1
        if !ERRORLEVEL! EQU 0 (
            echo [*] Backend server is ready!
            goto START_WALLET_WEB
        )
    ) else (
        REM curl not available, just check if port is listening
        echo [*] Backend server is ready! - curl not available, skipping health check
        goto START_WALLET_WEB
    )
)
set /A RETRY+=1
if !RETRY! LSS 30 (
    echo [*] Waiting... !RETRY!/30
    goto WAIT_BACKEND
)
echo [WARNING] Backend might not be ready yet, continuing anyway...

:START_WALLET_WEB
echo.
echo [2/3] Starting wallet-web on http://localhost:8080...

REM Use trunk serve in watch mode if available (for auto-reload on code changes)
REM Use goto-based approach to avoid if statement parsing issues
set USE_TRUNK=!TRUNK_AVAILABLE!

REM Check value and jump to appropriate label
if "!USE_TRUNK!" NEQ "1" goto :NO_TRUNK

REM Trunk is available - start it
echo [*] Starting Trunk in watch mode (auto-reload enabled)...
echo [*] Trunk will watch for changes and rebuild automatically
echo [*] Changes to src/, style/, or index.html will trigger rebuild
cd wallet-web
REM Start trunk serve in a new window - it will watch and rebuild automatically
start "TrunkWatch" cmd /k "trunk serve --port 8080 --address 127.0.0.1"
cd ..
echo [*] Trunk serve started in watch mode (check the TrunkWatch window for build output)
goto WAIT_WALLET

:NO_TRUNK

REM Fallback to wallet-server if trunk not available
if exist wallet-web\dist (
    echo [*] Starting wallet-server - serving built frontend...
    REM Always use cargo run to ensure latest build is used
    set SCRIPT_DIR=%~dp0
    echo @echo off > temp_wallet.bat
    echo cd /d "%SCRIPT_DIR%wallet-web" >> temp_wallet.bat
    echo cargo run --release --bin wallet-server >> temp_wallet.bat
    start "WalletServer" /MIN temp_wallet.bat
    timeout /t 1 /nobreak >nul 2>&1
    del temp_wallet.bat >nul 2>&1
) else (
    echo [WARNING] Frontend not built and Trunk not available. Skipping wallet-server startup.
    echo [WARNING] Install Trunk with: cargo install trunk
    echo [WARNING] Then run: cd wallet-web ^&^& trunk serve
    echo.
    goto START_TERMINAL
)

REM Wait for wallet-web to start
echo [*] Waiting for wallet-web to initialize...
set RETRY=0
:WAIT_WALLET
timeout /t 1 /nobreak >nul 2>&1
netstat -ano 2>nul | findstr :8080 | findstr LISTENING > nul
if !ERRORLEVEL! EQU 0 (
    REM Verify server responds
    where curl.exe >nul 2>nul
    if !ERRORLEVEL! EQU 0 (
        curl.exe -s http://127.0.0.1:8080/ >nul 2>&1
        if !ERRORLEVEL! EQU 0 (
            echo [*] Wallet-web is ready!
            goto START_TERMINAL
        )
    ) else (
        REM curl not available, just check if port is listening
        echo [*] Wallet-web is ready! - curl not available, skipping response check
        goto START_TERMINAL
    )
)
set /A RETRY+=1
if !RETRY! LSS 30 (
    echo [*] Waiting... !RETRY!/30
    goto WAIT_WALLET
)
echo [WARNING] Wallet-web might not be ready yet, continuing anyway...

:START_TERMINAL
echo.
echo [3/3] Starting GUI application with DEBUG mode...
echo.

REM Always start debug viewer if it exists
if exist debug-viewer.ps1 (
    echo [*] Starting debug viewer in new window...
    
    REM Ensure logs directory exists before starting viewer
    if not exist logs (
        echo [*] Creating logs directory...
        mkdir logs
    )
    
    REM Create empty log file if it doesn't exist (terminal will overwrite it)
    if not exist logs\debug-realtime.log (
        echo. > logs\debug-realtime.log
        echo [*] Created debug-realtime.log file for viewer tracking
    )
    
    REM Start debug viewer in a separate window that won't close on error
    start "XForce Terminal - Debug Viewer" powershell -NoExit -ExecutionPolicy Bypass -Command "& {try { . '%~dp0debug-viewer.ps1' } catch { Write-Host 'Debug viewer error:' $_.Exception.Message; Write-Host 'Press Enter to exit...'; Read-Host | Out-Null }}"
    timeout /t 3 /nobreak >nul
    echo [*] Debug viewer started successfully
    echo [*] Debug viewer is tracking logs\debug-realtime.log
) else (
    echo [WARNING] debug-viewer.ps1 not found - debug viewer will not be available
)
echo.

REM Final verification that backend is ready before starting terminal
echo [*] Verifying backend is ready...
set VERIFY_RETRY=0
where curl.exe >nul 2>nul
if errorlevel 1 goto SKIP_HEALTH_CHECK
goto VERIFY_BACKEND
:SKIP_HEALTH_CHECK
echo [*] Backend port is listening - curl not available, skipping health check...
echo [*] Proceeding to start terminal...
goto SHOW_STATUS
:VERIFY_BACKEND
curl.exe -s http://127.0.0.1:3001/health >nul 2>&1
if errorlevel 1 goto VERIFY_RETRY
echo [*] Backend verified and ready!
goto SHOW_STATUS
:VERIFY_RETRY
set /A VERIFY_RETRY+=1
if !VERIFY_RETRY! LSS 10 goto VERIFY_LOOP
echo [WARNING] Backend health check failed, but starting terminal anyway...
echo [WARNING] Terminal may not be able to connect to backend.
goto SHOW_STATUS
:VERIFY_LOOP
echo [*] Verifying... !VERIFY_RETRY! of 10
timeout /t 1 /nobreak >nul 2>&1
goto VERIFY_BACKEND

:SHOW_STATUS
echo.
echo ============================================
echo  Services Running:
echo ============================================
echo  Backend:     http://localhost:3001
if exist wallet-web\dist (
    echo  Wallet-Web:  http://localhost:8080
) else (
    echo  Wallet-Web:  Not running (frontend not built)
)
echo  Program ID:  HS63bw1V1qTM5uWf92q3uaFdqogrc4SN9qUJSR8aqBMx
echo  Network:     Devnet
echo.
echo  DEBUG MODE ENABLED:
echo    - RUST_LOG=%RUST_LOG%
echo    - TERMINAL_DEBUG_REALTIME=%TERMINAL_DEBUG_REALTIME%
echo    - TERMINAL_DEBUG_UI=%TERMINAL_DEBUG_UI%
echo    - TERMINAL_DEBUG_CHARTS=%TERMINAL_DEBUG_CHARTS%
echo    - RUST_BACKTRACE=%RUST_BACKTRACE%
echo.
echo  Starting terminal GUI with debug logging...
echo  GUI window will open shortly...
echo  Press Ctrl+D in the terminal to toggle debug overlay
echo ============================================
echo.

REM Always use cargo run to ensure latest build is used
REM This ensures changes are applied even if binary exists
REM Create temp batch file with all debug environment variables
if "%TERMINAL_BUILD_FAILED%"=="1" (
    echo [WARNING] Skipping terminal - build failed earlier
    echo [WARNING] Backend is running on http://localhost:3001
    echo [WARNING] You can build and run terminal separately
) else (
    echo [*] Starting terminal with latest build and debug environment...
    set SCRIPT_DIR=%~dp0
    echo @echo off > temp_terminal.bat
    echo REM Auto-generated by start-with-debug.bat >> temp_terminal.bat
    echo REM Set all debug environment variables >> temp_terminal.bat
    echo set RUST_LOG=%RUST_LOG% >> temp_terminal.bat
    echo set RUST_BACKTRACE=%RUST_BACKTRACE% >> temp_terminal.bat
    echo set TERMINAL_DEBUG_REALTIME=%TERMINAL_DEBUG_REALTIME% >> temp_terminal.bat
    echo set TERMINAL_TRACE_ENABLED=%TERMINAL_TRACE_ENABLED% >> temp_terminal.bat
    echo set TERMINAL_FREEZE_THRESHOLD=%TERMINAL_FREEZE_THRESHOLD% >> temp_terminal.bat
    echo set TERMINAL_DEBUG_UI=%TERMINAL_DEBUG_UI% >> temp_terminal.bat
    echo set TERMINAL_DEBUG_CHARTS=%TERMINAL_DEBUG_CHARTS% >> temp_terminal.bat
    echo cd /d "%SCRIPT_DIR%" >> temp_terminal.bat
    echo cargo run --release --bin terminal >> temp_terminal.bat
    echo if errorlevel 1 goto TERMINAL_ERROR >> temp_terminal.bat
    echo goto TERMINAL_SUCCESS >> temp_terminal.bat
    echo :TERMINAL_ERROR >> temp_terminal.bat
    echo echo. >> temp_terminal.bat
    echo echo [ERROR] Terminal exited with error code! >> temp_terminal.bat
    echo echo [ERROR] Check the output above for error messages. >> temp_terminal.bat
    echo echo [ERROR] Also check logs\debug-realtime.log for detailed logs. >> temp_terminal.bat
    echo echo. >> temp_terminal.bat
    echo pause >> temp_terminal.bat
    echo :TERMINAL_SUCCESS >> temp_terminal.bat
    REM Use call instead of start so we wait for terminal to exit
    call temp_terminal.bat
    REM Clean up temp file
    del temp_terminal.bat >nul 2>&1
)

REM Cleanup when terminal exits
echo.
echo [*] Shutting down servers...

REM Fallback to port-based cleanup
set CLEANUP_COUNT=0
for /f "tokens=5" %%a in ('netstat -ano 2^>nul ^| findstr :3001') do (
    taskkill /F /PID %%a >nul 2>&1
    if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1
)
for /f "tokens=5" %%a in ('netstat -ano 2^>nul ^| findstr :8080') do (
    taskkill /F /PID %%a >nul 2>&1
    if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1
)

REM Final cleanup by process name
taskkill /IM wallet-server.exe /F >nul 2>&1
if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1
taskkill /IM backend.exe /F >nul 2>&1
if !ERRORLEVEL! EQU 0 set /A CLEANUP_COUNT+=1

if !CLEANUP_COUNT! GTR 0 (
    echo [*] Stopped !CLEANUP_COUNT! server process(es)
) else (
    echo [*] No servers were running
)

echo.
echo ============================================
echo  GUI application closed. Goodbye!
echo ============================================
endlocal
