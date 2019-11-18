@echo off
set DIR=%~dp0
set PROJECT_PATH=%DIR%..\

set TARGET_PID=%1
set SAMPLE_INTERVAL=%2

if "%TARGET_PID%" == "" (
    echo "usage: start-trace-agent.bat <pid> [sample interval(ms)]"
    exit /b 1
)

if "%SAMPLE_INTERVAL%" == "" (
    set SAMPLE_INTERVAL=5
)

set ATTACHER_PATH=%PROJECT_PATH%\lib\flare-attacher-jar-with-dependencies.jar
set AGENT_PATH=%PROJECT_PATH%\lib\flareagent.dll
set AGENT_OPTIONS=trace=on,interval=%SAMPLE_INTERVAL%

echo "AGENT_PATH: %AGENT_PATH%"
echo "AGENT_OPTS: %AGENT_OPTS%"
echo "TARGET_PID: %TARGET_PID%"

if "%JAVA_HOME%" == "" {
    echo Required system env: JAVA_HOME
    exit /b 1
}
"%JAVA_HOME%\bin\java" -Xbootclasspath/a:%JAVA_HOME%/lib/tools.jar -jar %ATTACHER_PATH%  %AGENT_PATH%  %AGENT_OPTIONS% %TARGET_PID%

