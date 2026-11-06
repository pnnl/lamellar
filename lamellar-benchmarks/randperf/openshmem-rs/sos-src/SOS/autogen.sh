#! /bin/bash

test -d ./config || mkdir ./config

script_dir="$(cd "$(dirname "$0")" && pwd)"
current_dir="$(pwd)"

# Check if the script is being run from within the root SOS source directory
if [ "$script_dir" != "$current_dir" ]; then
    echo "Error: Please run ./autogen.sh from the top-level directory of SOS."
    exit 1
fi

autoreconf -vif
