@echo off
set DIR=%~dp0
set PROJECT_PATH=%DIR%..\..
set ATTACHER_PATH=%PROJECT_PATH%\flare-attacher\target\flare-attacher-jar-with-dependencies.jar
set AGENT_PATH=%PROJECT_PATH%\flare-agent\target\release\flareagent.dll
if "%1" == "debug" (
    set AGENT_PATH=%PROJECT_PATH%\flare-agent\target\debug\flareagent.dll
)
set AGENT_OPTIONS=trace=on,interval=5

%JAVA_HOME%\bin\java -Xbootclasspath/a:%JAVA_HOME%/lib/tools.jar -jar %ATTACHER_PATH%  %AGENT_PATH%  %AGENT_OPTIONS%

