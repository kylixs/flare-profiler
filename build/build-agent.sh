#!/bin/bash

REBUILD=false
if [[ "$1" == "clean" ]];then
    REBUILD=true
fi

#cargo params
export RUSTFLAGS=-Awarnings

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
DIST_DIR="$PROJECT_PATH/target/flare-profiler/agent"
BUILD_DIR="$PROJECT_PATH/flare-agent/target/release"

if [[ "$REBUILD" == "true" ]];then
    echo "cleaning flare-agent dist dir: $DIST_DIR .."
    rm -rf $DIST_DIR
    mkdir -p $DIST_DIR
    echo "cleaning flare-agent build dir: $BUILD_DIR .."
    rm -rf $BUILD_DIR
fi

#copy flare-agent assets files
echo "copy flare-agent assets files .."
cp -r $PROJECT_PATH/assets/agent/* $DIST_DIR/

#build flare-agent
echo "build flare-agent lib .."
cd $PROJECT_PATH/flare-agent
cargo build --release

#copy agent lib file
AGENT_LIB_FILE=""
if [[ -f "$BUILD_DIR/flareagent.dll" ]];then
    AGENT_LIB_FILE="$BUILD_DIR/flareagent.dll"
elif [[ -f "$BUILD_DIR/libflareagent.so" ]];then
    AGENT_LIB_FILE="$BUILD_DIR/libflareagent.so"
elif [[ -f "$BUILD_DIR/libflareagent.dylib" ]];then
    AGENT_LIB_FILE="$BUILD_DIR/libflareagent.dylib"
else
    echo "build failed, flare-agent lib file not found!"
    exit 1
fi

echo "copy flare-agent lib .."
cp $AGENT_LIB_FILE $DIST_DIR/lib/


#build flare-attacher
ATTACHER_DIR="$PROJECT_PATH/flare-attacher"

echo "copy flare-attacher lib .."
cp $ATTACHER_DIR/target/flare-attacher-jar-with-dependencies.jar $DIST_DIR/lib/