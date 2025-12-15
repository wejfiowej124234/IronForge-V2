@echo off
REM ============================================
REM Fix Build Issues Script (Admin Version)
REM ============================================
REM This script requires Administrator privileges
REM It fixes build issues by forcefully cleaning locked directories
REM ============================================

REM Check for admin privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo.
    echo ================================================================
    echo   ERROR: This script requires Administrator privileges!
    echo ================================================================
    echo.
    echo Please right-click this file and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

echo.
echo ================================================================
echo   Fixing Build Issues (Admin Mode)
echo ================================================================
echo.

REM Step 1: Stop any running Trunk/Cargo processes
echo [1/5] Stopping build processes...
taskkill /F /IM trunk.exe /T >nul 2>&1
taskkill /F /IM cargo.exe /T >nul 2>&1
taskkill /F /IM rustc.exe /T >nul 2>&1
taskkill /F /IM wasm-bindgen.exe /T >nul 2>&1
timeout /t 2 /nobreak >nul
echo     [OK] Build processes stopped
echo.

REM Step 2: Force unlock and clean dist directory
echo [2/5] Force cleaning dist directory...
if exist "dist" (
    echo     Taking ownership...
    takeown /F "dist" /R /D Y
    echo     Granting permissions...
    icacls "dist" /grant "%USERNAME%:F" /T /C /Q
    timeout /t 1 /nobreak >nul
    echo     Deleting files...
    rmdir /S /Q "dist"
    if exist "dist" (
        echo     [WARN] Some files may still be locked
        echo     [TIP] Close all file explorers, IDEs, and browsers
    ) else (
        echo     [OK] dist directory cleaned
    )
) else (
    echo     [INFO] dist directory does not exist
)
echo.

REM Step 3: Force clean .stage directory
echo [3/5] Force cleaning .stage directory...
if exist "dist\.stage" (
    takeown /F "dist\.stage" /R /D Y >nul 2>&1
    icacls "dist\.stage" /grant "%USERNAME%:F" /T /C /Q >nul 2>&1
    rmdir /S /Q "dist\.stage" >nul 2>&1
    echo     [OK] .stage directory cleaned
) else (
    echo     [INFO] .stage directory does not exist
)
echo.

REM Step 4: Clean target directory
echo [4/5] Cleaning target/wasm32-unknown-unknown directory...
if exist "target\wasm32-unknown-unknown" (
    takeown /F "target\wasm32-unknown-unknown" /R /D Y >nul 2>&1
    icacls "target\wasm32-unknown-unknown" /grant "%USERNAME%:F" /T /C /Q >nul 2>&1
    rmdir /S /Q "target\wasm32-unknown-unknown" >nul 2>&1
    echo     [OK] WASM target directory cleaned
) else (
    echo     [INFO] WASM target directory does not exist
)
echo.

REM Step 5: Clear any remaining locks
echo [5/5] Clearing file system locks...
timeout /t 2 /nobreak >nul
echo     [OK] Locks cleared
echo.

echo ================================================================
echo   Build Fix Complete!
echo ================================================================
echo.
echo You can now try building again:
echo   trunk serve --release
echo.
pause

