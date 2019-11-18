#!/bin/bash

#cargo params
export RUSTFLAGS=-Awarnings

PROJECT_PATH="$(cd "$(dirname $0)"; pwd -P )"

#build flare-server
$PROJECT_PATH/build/build-server.sh $@

#build flare-agent
$PROJECT_PATH/build/build-agent.sh $@

#build flare-ui
#$PROJECT_PATH/build/build-ui.sh $@

echo "build flare-profiler finished: $PROJECT_PATH/target/flare-profiler"

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
fi
cd $PROJECT_PATH
echo "packaged: $(find $PROJECT_PATH/target -name flare-profiler-*.tar.gz)"
