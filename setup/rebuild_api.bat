@echo off
REM Windows Build Script with Automatic Process Cleanup
REM Solves: "Access is denied" errors when rebuilding intent-api.exe
REM Usage: setup\rebuild_api.bat [--release]

echo [REBUILD] Starting Windows build with process cleanup...

REM Kill cargo run processes
echo [CLEANUP] Terminating cargo run processes...
for /f "tokens=2" %%i in ('tasklist /FI "IMAGENAME eq cargo.exe" /NH 2^>nul') do (
    taskkill /F /PID %%i >nul 2>&1
)

REM Kill intent-api.exe processes
echo [CLEANUP] Terminating intent-api.exe processes...
taskkill /F /IM intent-api.exe >nul 2>&1

REM Give Windows time to release file locks
timeout /t 2 /nobreak >nul

REM Build based on argument
if "%1"=="--release" (
    echo [BUILD] Building release version...
    cargo build --release --bin intent-api
) else (
    echo [BUILD] Building debug version...
    cargo build --bin intent-api
)

if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Build completed successfully
) else (
    echo [ERROR] Build failed with exit code %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)
