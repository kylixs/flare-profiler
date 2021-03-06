@echo off

REM DON'T CHANGE THE FIRST LINE OF THE FILE, WINDOWS SERVICE RUN BAT NEED IT! (@echo off) 
REM don't call 'echo' before 'start xxx.bat'
REM You can specify Java Home via FLARE_JAVA_HOME here or Windows System Environment, but not in cmd.exe
REM set FLARE_JAVA_HOME=C:\Program Files\Java\jdk1.8.0_131

set basedir=%~dp0
set filename=%~nx0
set srv_name=FlareAgent
set agent_port=3333
set interval=5

REM parse extend args
set arg1=%1
set pid=
set port=
set ignoreTools=0
set flare_remove_srv=0
set flare_service=0
set srv_interact=0
for %%a in (%*) do (
  if "%%a"=="--remove" set flare_remove_srv=1
  if "%%a"=="--service" set flare_service=1
  if "%%a"=="--interact"  set srv_interact=1
  if "%%a"=="--ignore-tools" set ignoreTools=1
)


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

REM rename option name '-' to '_'
if not "%agent-port%"=="" set agent_port=%agent-port%
REM if not "%interval%"=="" set interval=%interval%


REM Setup JAVA_HOME
REM Decode -java-home: '@' -> ' '
if not "%java-home%"=="" set JAVA_HOME=%java-home:@= %
REM If has FLARE_JAVA_HOME, overriding JAVA_HOME
if not "%FLARE_JAVA_HOME%" == "" set JAVA_HOME=%FLARE_JAVA_HOME%
REM use defined is better then "%var%" == "", avoid trouble of ""
if not defined JAVA_HOME goto noJavaHome
REM Remove "" in path
set JAVA_HOME=%JAVA_HOME:"=%
if not exist "%JAVA_HOME%\bin\java.exe" goto noJavaHome
if %ignoreTools% == 1 (
  echo Ignore tools.jar, make sure the java version ^>^= 9
) else (
  if not exist "%JAVA_HOME%\lib\tools.jar" (
    echo Can not find lib\tools.jar under %JAVA_HOME%!
    echo If java version ^<^= 1.8, please make sure JAVA_HOME point to a JDK not a JRE.
    echo If java version ^>^= 9, try to run %filename% ^<pid^> --ignore-tools
    goto :end
  )
)
set JAVACMD="%JAVA_HOME%\bin\java"

REM Runas Service, don't call 'echo' before 'start xxx.bat'
set flare_args=-agent-port %agent_port% 
if %srv_interact%==0  set flare_args=%flare_args% --no-interact
if %ignoreTools%==1 set flare_args=%flare_args% --ignore-tools
if %flare_service%==1 (
	REM run xxx.bat
	start /wait %basedir%\stop-agent.bat %pid% %flare_args%
	exit 0
	
	REM DEBUG run args
	REM echo flare_args: %flare_args%
	REM echo start /wait %basedir%\xxx.bat %pid% %flare_args%
	REM exit /b 0
)

REM If the first arg is a number, then set it as pid
echo %arg1%| findstr /r "^[1-9][0-9]*$">nul
if %errorlevel% equ 0  set pid=%arg1%

echo pid: %pid%
echo port: %port%

if not ["%pid%"] == [""] (
    goto :prepare_srv
)
if not ["%port%"] == [""] (
    goto :find_port
)
if %flare_remove_srv%==1 (
    goto :remove_srv
)
goto :usage


:remove_srv
echo Removing service: %srv_name% ...
sc stop %srv_name%
sc delete %srv_name%
exit /b 0


:find_port
@rem find pid by port
echo %port%| findstr /r "^[1-9][0-9]*$">nul
if %errorlevel% neq 0 (
    echo port is not valid number!
    goto :usage
)

echo Finding process of listening on port: %port%
set query_pid_command='netstat -ano ^^^| findstr ":%port%" ^^^| findstr "LISTENING"'
set pid=
for /f "tokens=5" %%i in (%query_pid_command%) do (
    set pid=%%i
)
if "%pid%" == "" (
    echo None process listening on port: %port%
    goto :end
)
echo Target process pid is %pid%


:prepare_srv
REM validate pid
echo %pid%| findstr /r "^[1-9][0-9]*$">nul
if %errorlevel% neq 0 (
    echo PID is not valid number!
    goto :usage
)
echo Preparing flare agent service and injecting flare agent to process: %pid% ...

REM encode java path, avoid space in service args: ' ' -> '@'
set srv_java_home=-java-home %JAVA_HOME: =@%
set srv_args=-pid %pid% %srv_java_home% 
if %srv_interact%==1 (
	REM start as interact service
	sc start UI0Detect
	set srv_type=type= interact type= own
	set srv_binpath=binPath= "%basedir%\%filename% %srv_args% --service --interact"
) else (
	set srv_type=type= own
	set srv_binpath=binPath= "%basedir%\%filename% %srv_args% --service"
)
echo flareagent srv type: %srv_type%
echo flareagent srv binPath: %srv_binpath%

sc create %srv_name% start= demand %srv_type% %srv_binpath%
sc config %srv_name% start= demand %srv_type% %srv_binpath%
if %errorlevel% NEQ 0 (
	echo Config FlareAgent service failed
	exit /b -1
)

sc stop %srv_name%
REM fork start FlareAgent service, avoid blocking
if %srv_interact%==1 (
	start /B sc start %srv_name%
)else (
	start /B sc start %srv_name% > nul 2>&1
)

REM check and connect flareagent ..
echo Waitting for flare agent ...
set count=0


:waitfor_loop
echo checking
netstat -nao |findstr LIST |findstr :%agent_port%
IF %ERRORLEVEL% == 0 (
    set /a count+=1
    if %count% geq 8 (
        echo Flare agent port is listening, maybe inject failed.
        goto :end
    )
    ping -w 1 -n 2 0.0.0.0 > nul
    goto :waitfor_loop
)
echo Flare agent port is closed.
goto :end


:usage
echo FlareAgent for Windows Service.
echo Usage:
echo   %filename% java_pid [option] ..
echo   %filename% [-pid java_pid] [option] ..
echo   %filename% [-port java_port] [option] ..
echo(
echo Options:
echo   -pid java_pid      : Attach by java process pid
echo   -port java_port    : Attach by java process listen port
echo   -interval sample_interval_ms   : Change flare agent sample interval ms(default: 5) 
@rem echo   -agent-port port   : Change flare agent port 
echo   --interact         : Enable windows service interactive UI, useful for debug
echo   --remove           : Remove FlareAgent windows service
echo   --ignore-tools     : Ignore checking JAVA_HOME\lib\tools.jar for jdk 9/10/11
echo(
echo Example:
echo   %filename% 2351 
@rem echo   %filename% 2351 -agent-port 2000
echo   %filename% -pid 2351 --interact 
echo   %filename% -port 8080 --interact 
echo   %filename% --remove  #remove service
exit /b -1


:noJavaHome
echo JAVA_HOME: %JAVA_HOME%
echo The JAVA_HOME environment variable is not defined correctly.
echo It is needed to run this program.
echo NB: JAVA_HOME should point to a JDK not a JRE.
exit /b -1

:end
