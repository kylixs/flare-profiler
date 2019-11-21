#!/bin/bash

#cargo params
export RUSTFLAGS=-Awarnings

PROJECT_PATH="$(cd "$(dirname $0)"; pwd -P )"

#build flare-server
$PROJECT_PATH/scripts/build-server.sh $@
if [[ $? != 0 ]];then
   echo "exec build-server.sh failed."
   exit 1
fi

#build flare-agent
$PROJECT_PATH/scripts/build-agent.sh $@
if [[ $? != 0 ]];then
   echo "exec build-agent.sh failed."
   exit 1
fi

#build flare-ui
#$PROJECT_PATH/scripts/build-ui.sh $@
#if [[ $? != 0 ]];then
#   echo "exec build-ui.sh failed."
#   exit 1
#fi

echo "build finished: $PROJECT_PATH/target/flare-profiler"

#package
echo "packaging ..."
cd $PROJECT_PATH/target
if [[ "$OSTYPE" == "linux-gnu" ]]; then
    tar --exclude=flare-samples --exclude=*.bat -czf flare-profiler-linux.tar.gz flare-profiler
elif [[ "$OSTYPE" == "darwin"* ]]; then
    tar --exclude=flare-samples --exclude=*.bat -czf flare-profiler-macos.tar.gz flare-profiler
elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" ]]; then
    tar --exclude=flare-samples --exclude=*.sh -czf flare-profiler-windows.tar.gz  flare-profiler
else
    echo "Unknown OS type: $OSTYPE, ignore packaging."
    exit 1
fi
cd $PROJECT_PATH
echo "packaged: $(find $PROJECT_PATH/target -name flare-profiler-*.tar.gz)"
