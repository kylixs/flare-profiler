#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"

# GOTO for bash, based upon https://stackoverflow.com/a/31269848/5353461
function goto
{
 local label=$1
 cmd=$(sed -En "/^[[:space:]]*#[[:space:]]*$label:[[:space:]]*#/{:a;n;p;ba};" "$0")
 eval "$cmd"
 exit
}


USER_ACCOUNT="$1"
if [[ "$USER_ACCOUNT" == "" ]];then
    echo "required username"
    goto usage
fi
shift;

#TARGET_PID="$1"
#if [[ "$TARGET_PID" -eq "" ]];then
#    echo "required target java pid"
#    goto usage
#fi
#shift;

ACTION_SCRIPT="start-agent.sh"

#may not have permission to create log dir
#/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "$PROJECT_PATH/bin/$ACTION_SCRIPT $* > $PROJECT_PATH/logs/flare_agent_stdout.log 2>&1  & "
/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "$PROJECT_PATH/bin/$ACTION_SCRIPT $* & "
goto end


# usage: #
echo "Usage: $0 <username> <pid> [options]"
echo "Options:"
echo "     -interval <sample interval>    # sample interval(ms), default value is 5"
echo "     -address  <agent address>      # agent bind address, default value is 0.0.0.0:3333"
exit 1

# end: #
