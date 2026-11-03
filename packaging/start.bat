@echo off
setlocal
cd /d "%~dp0"

if not exist "listquiz.exe" (
    echo [ListQuiz] listquiz.exe is missing next to this file.
    pause
    exit /b 1
)

echo [ListQuiz] Starting server at http://localhost:8080
echo [ListQuiz] Keep this window open. Press Ctrl+C to stop.
start "" /b powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Sleep -Seconds 2; Start-Process 'http://localhost:8080'"

"%~dp0listquiz.exe"
set "exit_code=%ERRORLEVEL%"

if not "%exit_code%"=="0" (
    echo.
    echo [ListQuiz] Exited with code %exit_code%.
    pause
)

endlocal & exit /b %exit_code%
