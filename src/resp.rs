use std::{
    io::{BufRead, BufReader, Error, Read},
    net::TcpStream,
    process::exit,
};

const STRING: u8 = b'+';
const ERROR: u8 = b'-';
const INTEGER: u8 = b':';
const BULK: u8 = b'$';
const ARRAY: u8 = b'*';

#[derive(Debug)]
pub struct Value {
    typ: String,
    str: String,
    num: usize,
    bulk: String,
    arr: Vec<Value>,
}

pub struct Reader<'a> {
    stream: &'a mut BufReader<TcpStream>,
}

#[derive(Debug)]
pub struct ReadResult {
    something: u8,
}

impl Reader<'_> {
    pub fn read(&mut self) -> Value {
        let mut bytes = self.stream.bytes();
        let typ = bytes
            .nth(0)
            .expect("failed to read first byte 1")
            .expect("failed to read first byte 2");

        match typ {
            BULK => self.read_bulk(),
            _ => panic!("unknown type"),
        }
    }

    fn read_bulk(&mut self) -> Value {
        let len = self.read_integer();

        let mut str_buffer = vec![0; len];
        self.stream
            .read_exact(&mut str_buffer)
            .expect("failed to read string, invalid length");

        let read_string = std::str::from_utf8(&str_buffer).expect("failed to convert to string");

        println!("{:?}", read_string);

        Value {
            typ: "bulk".to_string(),
            str: "".to_string(),
            num: 0,
            bulk: read_string.to_string(),
            arr: vec![],
        }
    }

    fn read_integer(&mut self) -> usize {
        let mut buffer = Vec::with_capacity(256);
        let read_bytes = self
            .stream
            .read_until(b'\n', &mut buffer)
            .expect("failed to read until");

        if buffer[read_bytes - 1] != b'\n' && buffer[read_bytes - 2] != b'\r' {
            panic!("invalid string");
        }

        let s = std::str::from_utf8(&buffer[0..(read_bytes - 2)]).expect("failed to parse string");
        let len = s
            .parse::<usize>()
            .expect("failed to parse length, invalid value");

        len
    }

    pub fn read_line(&mut self) -> ReadResult {
        let mut bytes = self.stream.bytes();
        ReadResult { something: 1 }
    }

    pub fn new(stream: &mut BufReader<TcpStream>) -> Reader {
        Reader { stream }
    }
}
