#!/bin/bash

REBUILD=false
if [[ "$1" == "clean" ]];then
    REBUILD=true
fi

#cargo params
export RUSTFLAGS=-Awarnings

PROJECT_PATH="$(cd "$(dirname $0)/.."; pwd -P )"
DIST_DIR="$PROJECT_PATH/target/flare-profiler"
BUILD_DIR="$PROJECT_PATH/flare-ui/renderer/dist"

if [[ "$REBUILD" == "true" ]];then
    echo "cleaning flare-ui dist dir: $DIST_DIR .."
    rm -rf $DIST_DIR/res/static
    rm -rf $DIST_DIR/app
    mkdir -p $DIST_DIR/res/static
    mkdir -p $DIST_DIR/app
    echo "cleaning flare-ui build dir: $BUILD_DIR .."
    rm -rf $BUILD_DIR
fi

#copy assets files
#echo "copy flare-server assets files .."
#cp -r $PROJECT_PATH/assets/bin/* $DIST_DIR/


#build flare-server
echo "build flare-ui .."
cd $PROJECT_PATH/flare-ui/renderer
cnpm install && npm run build

#copy dist files
if [[ ! -f $BUILD_DIR/index.html ]];then
    echo "build flare-ui failed, index.html not found!"
    exit 1
fi
cp -r $BUILD_DIR/* $DIST_DIR/res/static/
