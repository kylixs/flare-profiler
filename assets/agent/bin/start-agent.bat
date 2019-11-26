@echo off
set DIR=%~dp0
set PROJECT_PATH=%DIR%..\

echo %1| findstr /r "^[1-9][0-9]*$">nul
if %errorlevel% equ 0  set PID=%1

REM Parse command line args (https://stackoverflow.com/a/35445653)
:read_params
if not %1/==/ (
    if not "%__var%"=="" (
        if not "%__var:~0,1%"=="-" (
            endlocal
            goto read_params
        )
        endlocal & set %__var:~1%=%~1
    ) else (
        setlocal & set __var=%~1
    )
    shift
    goto read_params
)

if "%INTERVAL%" == "" (
    set INTERVAL=5
)

if "%ADDRESS%" == "" (
    set ADDRESS=3333
)

if "%PID%" == "" (
    echo Required target java process pid
    goto usage
)


set ATTACHER_PATH=%PROJECT_PATH%\lib\flare-attacher-jar-with-dependencies.jar
set AGENT_PATH=%PROJECT_PATH%\lib\flareagent.dll
set AGENT_OPTS=trace=on,interval=%INTERVAL%,address=%ADDRESS%

echo "AGENT_PATH: %AGENT_PATH%"
echo "AGENT_OPTS: %AGENT_OPTS%"
echo "PID: %PID%"

if "%JAVA_HOME%" == "" (
    echo Required system env: JAVA_HOME
    exit /b 1
)
"%JAVA_HOME%\bin\java" -Xbootclasspath/a:"%JAVA_HOME%/lib/tools.jar" -jar %ATTACHER_PATH%  %AGENT_PATH%  %AGENT_OPTS% %PID%
goto end


:usage
echo Usage: start-trace-agent.bat ^<pid^> ^[options^]
echo Options:
echo      -interval ^<sample interval^>    # sample interval(ms), default value is 5
echo      -address  ^<agent address^>      # agent bind address, default value is 0.0.0.0:3333
exit /b 1

:end
