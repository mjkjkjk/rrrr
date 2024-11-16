#!/bin/bash

# Function to run a command and check its output
check_command() {
    local cmd=$1
    local expected=$2
    local result=$(redis-cli -p 6379 $cmd)
    
    if [ "$result" != "$expected" ]; then
        echo "✗ Error testing '$cmd'"
        echo "  Expected: '$expected'"
        echo "  Got:      '$result'"
        exit 1
    fi
}

run_command() {
    local cmd=$1
    local result=$(redis-cli -p 6379 $cmd)
}