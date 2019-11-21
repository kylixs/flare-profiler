#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"

USER_ACCOUNT="$1"
ACTION="$2"
TARGET_PID="$3"

if [[ "$USER_ACCOUNT" == "" ]];then
    echo "usage: $0 <username> <pid>"
    exit 1
fi

ACTION_SCRIPT="stop-trace-agent.sh"

if [[ "$TARGET_PID" == ""  ]];then
    echo "TARGET_PID is required"
    exit 1
fi

#may not have permission to create log dir
#/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "$PROJECT_PATH/bin/$ACTION_SCRIPT $TARGET_PID > $PROJECT_PATH/logs/flare_agent_stdout.log 2>&1  & "
/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "$PROJECT_PATH/bin/$ACTION_SCRIPT $TARGET_PID & "
