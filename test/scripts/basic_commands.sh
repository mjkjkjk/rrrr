#!/bin/bash

# Function to run a command and check its output
check_command() {
    local cmd=$1
    local expected=$2
    local result=$(redis-cli -p 6379 $cmd)
    
    echo "Testing '$cmd'"
    if [ "$result" = "$expected" ]; then
        echo "✓ Success: Got expected response '$expected'"
    else
        echo "✗ Error: Expected '$expected' but got '$result'"
        exit 1
    fi
}

# Run tests
check_command "PING" "PONG"
check_command "SET mykey myvalue" "OK"
check_command "GET mykey" "myvalue"
check_command "SET mykey2 1" "OK"
check_command "GET mykey2" "1"
check_command "INCRBY mykey2 1" "2"
check_command "GET mykey2" "2"
check_command "INCR mykey2" "3"
check_command "DECRBY mykey2 1" "2"
check_command "GET mykey2" "2"
check_command "DECR mykey2" "1"
check_command "INCR nonexistent" "1"