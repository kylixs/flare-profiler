#!/bin/bash

REBUILD=false
if [[ "$1" == "clean" ]];then
    REBUILD=true
fi

#cargo params
#compile with crt-static, making bin file without depends vcruntime dll
export RUSTFLAGS="-Awarnings -C target-feature=+crt-static"

if [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" ]]; then
  CARGO_OPTS="--target x86_64-pc-windows-msvc"
  TARGET_PATH="x86_64-pc-windows-msvc/release"
else
  CARGO_OPTS=""
  TARGET_PATH="release"
fi

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
DIST_DIR="$PROJECT_PATH/target/flare-profiler/agent"
BUILD_DIR="$PROJECT_PATH/flare-agent/target/$TARGET_PATH"

if [[ "$REBUILD" == "true" ]];then
    echo "cleaning flare-agent dist dir: $DIST_DIR .."
    rm -rf $DIST_DIR
    echo "cleaning flare-agent build dir: $BUILD_DIR .."
    rm -rf $BUILD_DIR
fi
mkdir -p $DIST_DIR

#copy flare-agent assets files
echo "copy flare-agent assets files .."
cp -r $PROJECT_PATH/assets/agent/* $DIST_DIR/

#build flare-agent
echo "build flare-agent lib .."
cd $PROJECT_PATH/flare-agent
cargo build  $CARGO_OPTS --release
if [[ $? != 0 ]];then
   echo "build flare agent failed."
   exit 1
fi

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
cd $ATTACHER_DIR
mvn package
echo "copy flare-attacher lib .."
cp $ATTACHER_DIR/target/flare-attacher-jar-with-dependencies.jar $DIST_DIR/lib/
