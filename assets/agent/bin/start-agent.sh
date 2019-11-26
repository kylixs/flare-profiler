#!/bin/bash

# GOTO for bash, based upon https://stackoverflow.com/a/31269848/5353461
function goto
{
 local label=$1
 cmd=$(sed -En "/^[[:space:]]*#[[:space:]]*$label:[[:space:]]*#/{:a;n;p;ba};" "$0")
 eval "$cmd"
 exit
}

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
echo "PROJECT_PATH:$PROJECT_PATH"

#check number (https://stackoverflow.com/a/806923/11712121)
re='^[0-9]+$'
if [[ $1 =~ $re ]] ; then
    PID="$1"
fi

#parse command line args(https://stackoverflow.com/a/14203146/11712121)
POSITIONAL=()
while [[ $# -gt 0 ]]
do
key="$1"

case $key in
    -p|-pid)
    PID="$2"
    shift # past argument
    shift # past value
    ;;
    -i|-interval)
    INTERVAL="$2"
    shift # past argument
    shift # past value
    ;;
    -a|-address)
    ADDRESS="$2"
    shift # past argument
    shift # past value
    ;;
    *)    # unknown option
    POSITIONAL+=("$1") # save it in an array for later
    shift # past argument
    ;;
esac
done
set -- "${POSITIONAL[@]}" # restore positional parameters

if [[ "$PID" -eq "" ]];then
    goto usage
fi

if [[ "$INTERVAL" == "" ]];then
    INTERVAL=5
fi

if [[ "$ADDRESS" == "" ]];then
    ADDRESS=3333
fi

LIB_SUFFIX=".so"
IS_MAC_OSX=$(uname -a | grep -i darwin)
if [[ "$IS_MAC_OSX" != ""  ]];then
  LIB_SUFFIX=".dylib"
fi

ATTACHER_PATH=$PROJECT_PATH/lib/flare-attacher-jar-with-dependencies.jar
AGENT_PATH=$PROJECT_PATH/lib/libflareagent$LIB_SUFFIX

AGENT_OPTS="trace=on,interval=$INTERVAL,address=$ADDRESS"

echo "AGENT_PATH: $AGENT_PATH"
echo "AGENT_OPTS: $AGENT_OPTS"
echo "PID: $PID"

if [[ "$JAVA_HOME" == ""  ]];then
  echo "Required system env: JAVA_HOME"
  exit 1
fi

$JAVA_HOME/bin/java -Xbootclasspath/a:$JAVA_HOME/lib/tools.jar -jar $ATTACHER_PATH  $AGENT_PATH $AGENT_OPTS $PID
goto end

# usage: #
echo "Usage: ./start-agent.sh <pid> [options]"
echo "Options:"
echo "     -interval <sample interval>    # sample interval(ms), default value is 5"
echo "     -address  <agent address>      # agent bind address, default value is 0.0.0.0:3333"
exit 1

# end: #
