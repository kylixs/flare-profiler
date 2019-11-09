#!/bin/bash

USER_ACCOUNT="szjls_gordian_ctp"
ACTION="$1"
ACTION_SCRIPT=""
if [[ "$ACTION" == "start" ]];then
    ACTION_SCRIPT="start-trace-agent.sh"
elif [[ "$ACTION" == "stop" ]];then
    ACTION_SCRIPT="stop-trace-agent.sh"
else
    echo "usage: <start/stop> <pid>"
    exit 1
fi


TARGET_PID="$2"
if [[ "$TARGET_PID" == ""  ]];then
    echo "TARGET_PID is required"
    exit 1
fi

/usr/bin/su - $USER_ACCOUNT  -s /bin/sh -c "/application/flareagent/bin/$ACTION_SCRIPT $TARGET_PID > //application/flareagent/logs/flareagent_stdout.log 2>&1  & "

