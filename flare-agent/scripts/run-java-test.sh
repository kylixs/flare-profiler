#!/bin/bash


MAIN_CLASS=Test
LOG_FILE=tracelog-$MAIN_CLASS.txt

JAVA=$JAVA_HOME/bin/java

#$JAVA -agentpath:target/debug/libjvmti.so -cp target/classes $MAIN_CLASS > $LOG_FILE
$JAVA -agentpath:target/release/libjvmti.so -cp target/classes $MAIN_CLASS > $LOG_FILE
