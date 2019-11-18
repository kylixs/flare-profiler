#!/bin/bash

PROJECT_PATH="$(cd "$(dirname $0)"; pwd -P )"
echo "PROJECT_PATH:$PROJECT_PATH"

#start flare-server
echo "Starting flare-server ..."
$PROJECT_PATH/bin/flare_server
