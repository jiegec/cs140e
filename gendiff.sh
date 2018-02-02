#!/usr/bin/env bash

for dir in 0-blinky 1-shell os
do
    echo ${dir}
    cd ${dir}
    git diff >../${dir}.diff
    cd ..
done
