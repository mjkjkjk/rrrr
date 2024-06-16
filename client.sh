#!/bin/zsh

# simple ping
echo "+PING\r\n" | nc localhost 6379

# bulk string
echo "\$5\r\nHola!\r\n" | nc localhost 6379

# simple string
echo "+Ahoy!\r\n" | nc localhost 6379

# array of 2 bulk strings
echo "*2\r\n\$5\r\nhello\r\n\$6\r\nworld!\r\n" | nc localhost 6379
