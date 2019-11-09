@echo off
set DIR=%~dp0
set PROJECT_PATH=%DIR%..\
set ATTACHER_PATH=%PROJECT_PATH%\lib\flare-attacher-jar-with-dependencies.jar
set AGENT_PATH=%PROJECT_PATH%\lib\flareagent.dll
set AGENT_OPTIONS=trace=on,interval=5
if not "%1" == "" (
    set TARGET_PID=%1
)

"%JAVA_HOME%\bin\java" -Xbootclasspath/a:%JAVA_HOME%/lib/tools.jar -jar %ATTACHER_PATH%  %AGENT_PATH%  %AGENT_OPTIONS% %TARGET_PID%

