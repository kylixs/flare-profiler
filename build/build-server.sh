#!/bin/bash

REBUILD=false
if [[ "$1" == "clean" ]];then
    REBUILD=true
fi

#cargo params
#compile with crt-static, making bin file without depends vcruntime dll
export RUSTFLAGS="-Awarnings -C target-feature=+crt-static"
CARGO_TARGET="x86_64-pc-windows-msvc"

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
DIST_DIR="$PROJECT_PATH/target/flare-profiler"
BUILD_DIR="$PROJECT_PATH/flare-server/target/$CARGO_TARGET/release"

if [[ "$REBUILD" == "true" ]];then
    echo "cleaning flare-server dist dir: $DIST_DIR .."
    rm -rf $DIST_DIR
    echo "cleaning flare-server build dir: $BUILD_DIR .."
    rm -rf $BUILD_DIR
fi
mkdir -p $DIST_DIR

#copy server assets files
echo "copy flare-server assets files .."
cp -r $PROJECT_PATH/assets/* $DIST_DIR/

#build flare-server
echo "build flare-server .."
cd $PROJECT_PATH/flare-server
cargo build --target $CARGO_TARGET --release

#copy flare-server files
BIN_FILE=""
if [[ -f "$BUILD_DIR/flare_server.exe" ]];then
    BIN_FILE="$BUILD_DIR/flare_server.exe"
elif [[ -f "$BUILD_DIR/flare_server" ]];then
    BIN_FILE="$BUILD_DIR/flare_server"
else
    echo "build failed, dist file not found!"
    exit 1
fi

echo "copy flare-server files .."
cp $BIN_FILE $DIST_DIR/bin/


#copy simpleui
cp -r $PROJECT_PATH/flare-server/static/simpleui/* $DIST_DIR/res/static/
