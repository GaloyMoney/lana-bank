#!/bin/bash

echo "COMMITHASH=$(git rev-parse HEAD)" > repo/.build-args
