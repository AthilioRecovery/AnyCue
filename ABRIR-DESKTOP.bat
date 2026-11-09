@echo off
cd /d "%~dp0"
where node >nul 2>&1 || (echo Instale Node.js 22 ou superior. & pause & exit /b 1)
where cargo >nul 2>&1 || (echo Instale Rust e Visual Studio Build Tools C++ para compilar AnyCue. & pause & exit /b 1)
cd apps\desktop
call npm install || (pause & exit /b 1)
call npm run tauri dev
pause
