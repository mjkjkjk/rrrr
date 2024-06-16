#!/bin/zsh


# simple ping
OUTPUT="$(echo "+PING\r\n" | nc localhost 6379)"
expected=$'+PONG\r'
if [[ "$OUTPUT" != "$expected" ]] then 
    echo "PING invalid response: $OUTPUT";
    echo "Expected: $expected";
    echo "$OUTPUT" | hexdump;
    echo "$expected" | hexdump;
    exit 1
fi

# ping with single argument
OUTPUT="$(echo "+PING\r\n\$5\r\nHello\r\n" | nc localhost 6379)"
expected=$'$5\r\nHello\r'
if [[ "$OUTPUT" != "$expected" ]] then 
    echo "PING invalid response: $OUTPUT";
    echo "Expected: $expected";
    echo "$OUTPUT" | hexdump;
    echo "$expected" | hexdump;
    exit 1
fi

# ping with multiple arguments
