#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"

USER_ACCOUNT="$1"
if [[ "$USER_ACCOUNT" == "" ]];then
    echo "required username"
    echo "usage: $0 <username> <pid> [options]"
    exit 1
fi
shift;

TARGET_PID="$1"
if [[ "$TARGET_PID" -eq "" ]];then
    echo "required target java pid"
    echo "usage: $0 <username> <pid> [options]"
    exit 1
fi
shift;

ACTION_SCRIPT="start-agent.sh"

#may not have permission to create log dir
#/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "$PROJECT_PATH/bin/$ACTION_SCRIPT $TARGET_PID $@ > $PROJECT_PATH/logs/flare_agent_stdout.log 2>&1  & "
/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "$PROJECT_PATH/bin/$ACTION_SCRIPT $TARGET_PID $@ & "
