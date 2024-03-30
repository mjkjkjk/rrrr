use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    str,
};

fn handle_client(stream: &TcpStream) -> String {
    let mut buffered_reader = BufReader::new(stream);

    // get data type
    // expecting only string ($) at this time
    let mut value_type = [0];
    buffered_reader
        .read_exact(&mut value_type)
        .expect("failed to read first byte");

    match value_type[0] {
        b'$' => (),
        _ => panic!("Invalid type, expecting bulk strings only"),
    }

    // get length of string
    let mut buffer = Vec::with_capacity(256);
    let read_bytes = buffered_reader
        .read_until(b'\n', &mut buffer)
        .expect("failed to read until");
    if buffer[read_bytes - 1] != b'\n' && buffer[read_bytes - 2] != b'\r' {
        panic!("invalid string");
    }

    // parse length as usize
    let s = std::str::from_utf8(&buffer[0..(read_bytes - 2)]).expect("failed to parse string");
    let len = s
        .parse::<usize>()
        .expect("failed to parse length, invalid value");

    // write len bytes into buffer
    let mut str_buffer = vec![0; len];
    buffered_reader
        .read_exact(&mut str_buffer)
        .expect("failed to read string, invalid length");

    let read_string = std::str::from_utf8(&str_buffer).expect("failed to convert to string");

    // check again for \r\n
    let mut end_buffer = vec![0; 2];
    buffered_reader
        .read_exact(&mut end_buffer)
        .expect("failed to read end bytes");
    if end_buffer[1] != b'\n' && end_buffer[0] != b'\r' {
        panic!("invalid string");
    }

    read_string.to_string()
}

fn main() {
    println!("Hello, world!");

    /*
    fd = socket()
    bind(fd, address)
    listen(fd)
    while True:
        conn_fd = accept(fd)
        do_something_with(conn_fd)
        close(conn_fd)
     */
    let listener = TcpListener::bind("0.0.0.0:6379").expect("failed to bind");

    for stream in listener.incoming() {
        let mut s = stream.expect("failed to open stream");
        let result = handle_client(&s);
        println!("{}", result); // TODO result has no \0 at the end???
        s.write(result.as_bytes()).expect("failed to write back");
    }
}
