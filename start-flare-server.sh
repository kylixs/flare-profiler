#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)"; pwd -P )"

cd $PROJECT_PATH/flare-server
if [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" ]]; then
    ./target/x86_64-pc-windows-msvc/release/flare_server.exe
else
    ./target/release/flare_server 
fi

cd $PROJECT_PATH
