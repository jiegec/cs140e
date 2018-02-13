#!/usr/bin/env bash

for dir in 0-blinky 1-shell 2-fs os 
do
    echo ${dir}
    cd ${dir}
    git add -N .
    git diff >../${dir}.diff
    cd ..
done
