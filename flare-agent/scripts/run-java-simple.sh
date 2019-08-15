#!/bin/bash
PROJECT_DIR="$( cd "$(dirname "$0")"/.. ; pwd -P )"

echo "PROJECT_DIR: $PROJECT_DIR"

JAVA=$JAVA_HOME/bin/java
JAVA_CP=$PROJECT_DIR/target/classes
JVMTI_PATH=$PROJECT_DIR/target/release/libjvmti.so

#$JAVA -agentpath:$JVMTI_PATH -cp $JAVA_CP  Simple > tracelog.txt
$JAVA -agentpath:$JVMTI_PATH -cp $JAVA_CP  Simple > tracelog.txt

