#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
echo "PROJECT_PATH:$PROJECT_PATH"

TARGET_PID="$1"
SAMPLE_INTERVAL="$2"

if [[ "$TARGET_PID" -eq "" ]];then
    echo "usage: start-trace-agent.sh <pid> [sample interval(ms)]"
    exit 1
fi

if [[ "$SAMPLE_INTERVAL" == "" ]];then
    SAMPLE_INTERVAL=5
fi

LIB_SUFFIX=".so"
IS_MAC_OSX=$(uname -a | grep -i darwin)
if [[ "$IS_MAC_OSX" != ""  ]];then
  LIB_SUFFIX=".dylib"
fi

ATTACHER_PATH=$PROJECT_PATH/lib/flare-attacher-jar-with-dependencies.jar
AGENT_PATH=$PROJECT_PATH/lib/libflareagent$LIB_SUFFIX

AGENT_OPTS="trace=on,interval=$SAMPLE_INTERVAL"

echo "AGENT_PATH: $AGENT_PATH"
echo "AGENT_OPTS: $AGENT_OPTS"
echo "TARGET_PID: $TARGET_PID"

if [[ "$JAVA_HOME" == ""  ]];then
  echo "Required system env: JAVA_HOME"
  exit 1
fi

$JAVA_HOME/bin/java -Xbootclasspath/a:$JAVA_HOME/lib/tools.jar -jar $ATTACHER_PATH  $AGENT_PATH $AGENT_OPTS $TARGET_PID

