#!/bin/bash

#cargo params
export RUSTFLAGS=-Awarnings

PROJECT_PATH="$(cd "$(dirname $0)"; pwd -P )"

mkdir -p $PROJECT_PATH/target/flare-profiler

#build flare-server
$PROJECT_PATH/build/build-server.sh $@

#build flare-agent
$PROJECT_PATH/build/build-agent.sh $@

#build flare-ui
$PROJECT_PATH/build/build-ui.sh $@

echo "build flare-profiler finished: $PROJECT_PATH/target/flare-profiler"
