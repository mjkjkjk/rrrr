#!/bin/zsh

# simple ping
echo "+PING\r\n\0" | nc localhost 6379 -c

# bulk string
echo "\$5\r\nHola!\r\n" | nc localhost 6379 -c

# simple string
echo "+Ahoy!\r\n" | nc localhost 6379 -c

# array of 2 bulk strings
echo "*2\r\n\$5\r\nhello\r\n\$6\r\nworld!\r\n" | nc localhost 6379 -c
