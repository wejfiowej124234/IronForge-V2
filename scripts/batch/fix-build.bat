@echo off
REM ============================================
REM Fix Build Issues Script
REM ============================================
REM This script fixes common build issues:
REM 1. Cleans dist and .stage directories
REM 2. Stops any running build processes
REM 3. Removes file locks
REM ============================================

echo.
echo ================================================================
echo   Fixing Build Issues
echo ================================================================
echo.

REM Step 1: Stop any running Trunk/Cargo processes
echo [1/4] Stopping build processes...
taskkill /F /IM trunk.exe /T >nul 2>&1
taskkill /F /IM cargo.exe /T >nul 2>&1
taskkill /F /IM rustc.exe /T >nul 2>&1
timeout /t 2 /nobreak >nul
echo     [OK] Build processes stopped
echo.

REM Step 2: Clean dist directory (with retry and force unlock)
echo [2/4] Cleaning dist directory...
if exist "dist" (
    REM Try to unlock files first
    echo     Attempting to unlock files...
    takeown /F "dist" /R /D Y >nul 2>&1
    icacls "dist" /grant "%USERNAME%:F" /T /C /Q >nul 2>&1
    timeout /t 1 /nobreak >nul
    
    REM Try to delete
    rmdir /S /Q "dist" 2>nul
    timeout /t 1 /nobreak >nul
    
    REM If still exists, try again with force
    if exist "dist" (
        echo     Retrying with force unlock...
        for /f "delims=" %%i in ('dir /b /s /a-d "dist" 2^>nul') do (
            takeown /F "%%i" >nul 2>&1
            icacls "%%i" /grant "%USERNAME%:F" >nul 2>&1
            del /F /Q "%%i" >nul 2>&1
        )
        for /f "delims=" %%i in ('dir /b /s /ad "dist" 2^>nul') do (
            rd /S /Q "%%i" >nul 2>&1
        )
        rmdir /S /Q "dist" 2>nul
    )
    
    if exist "dist" (
        echo     [WARN] Could not delete dist directory (may be locked)
        echo     [TIP] Close any file explorers, IDEs, or browsers that may have dist open
        echo     [TIP] Try running this script as Administrator
    ) else (
        echo     [OK] dist directory cleaned
    )
) else (
    echo     [INFO] dist directory does not exist
)
echo.

REM Step 3: Clean .stage directory (with retry)
echo [3/4] Cleaning .stage directory...
if exist "dist\.stage" (
    REM Try to unlock files first
    takeown /F "dist\.stage" /R /D Y >nul 2>&1
    icacls "dist\.stage" /grant "%USERNAME%:F" /T /C /Q >nul 2>&1
    timeout /t 1 /nobreak >nul
    
    REM Try to delete
    rmdir /S /Q "dist\.stage" 2>nul
    timeout /t 1 /nobreak >nul
    
    if exist "dist\.stage" (
        echo     [WARN] Could not delete .stage directory (may be locked)
        echo     [TIP] Try running this script as Administrator
    ) else (
        echo     [OK] .stage directory cleaned
    )
) else (
    echo     [INFO] .stage directory does not exist
)
echo.

REM Step 4: Clean target directory (optional, but can help)
echo [4/4] Cleaning target/wasm32-unknown-unknown directory...
if exist "target\wasm32-unknown-unknown" (
    rmdir /S /Q "target\wasm32-unknown-unknown" 2>nul
    echo     [OK] WASM target directory cleaned
) else (
    echo     [INFO] WASM target directory does not exist
)
echo.

echo ================================================================
echo   Build Fix Complete!
echo ================================================================
echo.
echo You can now try building again:
echo   trunk serve --release
echo.
pause

