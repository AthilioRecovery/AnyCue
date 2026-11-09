@echo off
cd /d "%~dp0"
where node >nul 2>&1 || (echo Instale Node.js 22. & pause & exit /b 1)
where cargo >nul 2>&1 || (echo Instale Rust e MSVC Build Tools C++. & pause & exit /b 1)
cd apps\desktop
call npm install || (pause & exit /b 1)
call npm run tauri build -- --bundles nsis
if errorlevel 1 (echo Compilacao falhou. & pause & exit /b 1)
echo EXE portable: ..\..\target\release\anycue-desktop.exe
echo Instalador: ..\..\target\release\bundle\nsis\
pause
