#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)"; pwd -P )"

cd $PROJECT_PATH/flare-server
./target/x86_64-pc-windows-msvc/release/flare_server.exe

cd $PROJECT_PATH
