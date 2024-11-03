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

# Run tests
check_command "FLUSHALL" "OK"
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

check_command "FLUSHALL" "OK"
check_command "INCR nonexistent" "1"
check_command "SET mykey myvalue" "OK"
check_command "INCR mykey" "ERR value is not an integer or out of range"
check_command "MGET mykey" "myvalue"

check_command "FLUSHALL" "OK"
check_command "MGET k1" ""
check_command "GET k1" ""
check_command "SET k1 v1" "OK"
check_command "EXISTS k1" "1"
check_command "EXISTS k1 k1" "2"
check_command "EXISTS k1 k1 k2" "2"
check_command "EXISTS k2" "0"

check_command "FLUSHALL" "OK"
check_command "EXPIRE k1 10" "0"
check_command "SET k1 v1" "OK"
check_command "EXPIRE k1 10" "1"
check_command "TTL k1" "10"
check_command "EXPIRE k1 -100" "1"
check_command "GET k1" ""
