@echo off
set DIR=%~dp0
set PROJECT_PATH=%DIR%

@rem run flare-server
echo Starting flare-server ...
"%PROJECT_PATH%\bin\flare_server.exe"
