# Open Wallet-Web
Write-Host "============================================" -ForegroundColor Cyan
Write-Host " Opening Wallet-Web" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
Set-Location $scriptDir

# Check if trunk is installed
$trunkPath = Get-Command trunk -ErrorAction SilentlyContinue
if (-not $trunkPath) {
    Write-Host "[!] ERROR: Trunk is not installed" -ForegroundColor Red
    Write-Host "[!] Install with: cargo install trunk" -ForegroundColor Yellow
    Write-Host ""
    pause
    exit 1
}

# Start Trunk serve with hot reloading (watch mode)
Write-Host "[*] Starting Trunk serve with hot reloading..." -ForegroundColor Yellow
Write-Host "[*] This will watch for changes and automatically rebuild" -ForegroundColor Yellow
Write-Host "[*] Opening browser..." -ForegroundColor Yellow
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Yellow
Write-Host ""

# Ensure we're in the repo root first (for workspace detection)
Push-Location $repoRoot

# Clear NO_COLOR if set to '1' (Trunk expects 'true' or 'false')
if ($env:NO_COLOR -eq "1") {
    $env:NO_COLOR = $null
}

# Start trunk serve from wallet-web directory (this handles hot reloading)
Push-Location $scriptDir

# Trunk serve will automatically watch for changes, rebuild, and reload
# The --open flag opens the browser automatically
& trunk serve --port 8080 --address 127.0.0.1 --open

Pop-Location
Pop-Location

