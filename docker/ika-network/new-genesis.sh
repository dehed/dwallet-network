#!/bin/bash
# Copyright (c) Mysten Labs, Inc.
# SPDX-License-Identifier: BSD-3-Clause-Clear
#
# assumes ika cli installed (brew install ika or cargo build --bin ika)

cd genesis
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r requirements.txt

DIR="files"

if [ -d "$DIR" ]; then
    echo "Directory $DIR exists. Removing..."
    rm -r "$DIR"
fi

echo "Creating directory $DIR..."
mkdir "$DIR"
echo "$DIR directory created."


./generate.py --genesis-template compose-validators.yaml --target-directory "$DIR"
