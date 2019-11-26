#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
echo "PROJECT_PATH:$PROJECT_PATH"

TARGET_PID="$1"
if [[ "$TARGET_PID" -eq "" ]];then
    echo "usage: stop-agent.sh <pid>"
    exit 1
fi

LIB_SUFFIX=".so"
IS_MAC_OSX=$(uname -a | grep -i darwin)
if [[ "$IS_MAC_OSX" != ""  ]];then
  LIB_SUFFIX=".dylib"
fi

ATTACHER_PATH=$PROJECT_PATH/lib/flare-attacher-jar-with-dependencies.jar
AGENT_PATH=$PROJECT_PATH/lib/libflareagent$LIB_SUFFIX
if [[ "$1" -ne "" ]];then
    TARGET_PID="$1"
fi
AGENT_OPTS=trace=off

if [[ "$JAVA_HOME" == ""  ]];then
  echo "Required system env: JAVA_HOME"
  exit 1
fi

$JAVA_HOME/bin/java -Xbootclasspath/a:$JAVA_HOME/lib/tools.jar -jar $ATTACHER_PATH  $AGENT_PATH $AGENT_OPTS $TARGET_PID

