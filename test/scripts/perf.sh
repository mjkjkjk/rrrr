#!/bin/bash

source ./test/util.sh

sample_iteration() {
    run_command "FLUSHALL"
    run_command "PING"
    run_command "SET k1 v1"
    run_command "GET k1"
    run_command "SET k2 v2"
    run_command "GET k2"
    run_command "SET k3 v3"
    run_command "GET k3"
}

loops() {
    n=0; while [[ $n -lt 1000 ]]; do sample_iteration; n=$((n+1)); done
}

time loops
