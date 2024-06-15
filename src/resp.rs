use std::{
    io::{BufRead, BufReader, BufWriter, Read, Write},
    mem,
    ops::Deref,
};

#[derive(Debug)]
pub enum Value {
    ValueString(ValueString),
    ValueInteger(ValueInteger),
    ValueBulk(ValueBulk),
    ValueArray(ValueArray),
    ValueNull,
    ValueError(String),
}

#[derive(Debug)]
pub struct ValueString {
    pub str: String,
}

#[derive(Debug)]
pub struct ValueInteger {
    num: i64,
}

#[derive(Debug)]
pub struct ValueBulk {
    bulk: String,
}

#[derive(Debug)]
pub struct ValueArray {
    arr: Vec<Value>,
}

const STRING: u8 = b'+';
const ERROR: u8 = b'-';
const INTEGER: u8 = b':';
const BULK: u8 = b'$';
const ARRAY: u8 = b'*';

impl Value {
    pub fn marshall(self) -> Vec<u8> {
        match self {
            Value::ValueArray(arr) => arr.marshall(),
            Value::ValueBulk(bulk) => bulk.marshall(),
            Value::ValueString(str) => str.marshall(),
            Value::ValueInteger(int) => int.marshall(),
            Value::ValueNull => {
                let mut v = vec![];
                v.extend("$-1\r\n".as_bytes());
                v
            }
            Value::ValueError(s) => {
                let mut v = vec![];
                v.push(ERROR);
                v.extend(s.as_bytes());
                v.push(b'\r');
                v.push(b'\n');
                v
            }
        }
    }
}

impl ValueString {
    fn marshall(self) -> Vec<u8> {
        let mut v = vec![];
        v.push(STRING);
        v.extend(self.str.as_bytes());
        v.push(b'\r');
        v.push(b'\n');
        v
    }
}

impl ValueBulk {
    fn marshall(self) -> Vec<u8> {
        let mut v = vec![];
        v.push(BULK);
        v.extend(self.bulk.len().to_string().bytes());
        v.push(b'\r');
        v.push(b'\n');
        v.extend(self.bulk.as_bytes());
        v.push(b'\r');
        v.push(b'\n');
        v
    }
}

impl ValueInteger {
    fn marshall(self) -> Vec<u8> {
        let mut v = vec![];
        v.push(INTEGER);
        v.extend(self.num.to_string().bytes());
        v.push(b'\r');
        v.push(b'\n');
        v
    }
}

impl ValueArray {
    fn marshall(self) -> Vec<u8> {
        let mut v = vec![];
        v.push(ARRAY);
        v.extend(self.arr.len().to_string().bytes());
        v.push(b'\r');
        v.push(b'\n');
        for val in self.arr {
            v.extend(val.marshall())
        }
        v
    }
}

pub struct Writer<T: Write> {
    stream: BufWriter<T>,
}

impl<T: Write> Writer<T> {
    pub fn new(stream: BufWriter<T>) -> Writer<T> {
        Writer { stream }
    }

    pub fn write(mut self, v: Value) {
        self.stream.write(&v.marshall());
    }
}

pub struct Reader<'a, T: Read> {
    stream: &'a mut BufReader<&'a mut T>,
}

impl<T: Read> Reader<'_, T> {
    pub fn read(&mut self) -> Value {
        let mut bytes = self.stream.bytes();
        let typ = bytes
            .nth(0)
            .expect("failed to read first byte 1")
            .expect("failed to read first byte 2");

        match typ {
            BULK => self.read_bulk(),
            ARRAY => self.read_array(),
            STRING => self.read_string(),
            _ => {
                println!("{:?}", typ);
                panic!("unknown type");
            }
        }
    }

    fn read_array(&mut self) -> Value {
        let len = self.read_integer();

        let mut v: Vec<Value> = vec![];

        for _i in 0..len {
            let val = self.read();
            v.push(val);
        }

        Value::ValueArray(ValueArray { arr: v })
    }

    fn read_bulk(&mut self) -> Value {
        let len = self.read_integer();

        let mut str_buffer = vec![0; len];
        self.stream
            .read_exact(&mut str_buffer)
            .expect("failed to read string, invalid length");

        let read_string = std::str::from_utf8(&str_buffer).expect("failed to convert to string");

        // consume \r\n
        self.read_line();

        Value::ValueBulk(ValueBulk {
            bulk: read_string.to_string(),
        })
    }

    fn read_string(&mut self) -> Value {
        let mut buffer = vec![];
        self.stream
            .read_until(b'\r', &mut buffer)
            .expect("failed to read string, invalid length");

        buffer.pop();
        let read_string = std::str::from_utf8(&buffer).expect("failed to convert to string");

        // consume \r\n
        self.read_line();

        Value::ValueString(ValueString {
            str: read_string.to_string(),
        })
    }

    fn read_integer(&mut self) -> usize {
        let mut buffer = vec![];
        let read_bytes = self
            .stream
            .read_until(b'\n', &mut buffer)
            .expect("failed to read until");

        if buffer[read_bytes - 1] != b'\n' && buffer[read_bytes - 2] != b'\r' {
            panic!("invalid string");
        }

        let s = std::str::from_utf8(&buffer[0..(read_bytes - 2)]).expect("failed to parse string");
        let int = s
            .parse::<usize>()
            .expect("failed to parse length, invalid value");

        int
    }

    pub fn read_line(&mut self) -> String {
        let mut buffer = vec![];
        let read_bytes = self
            .stream
            .read_until(b'\n', &mut buffer)
            .expect("failed to read until");

        if buffer[read_bytes - 1] != b'\n' && buffer[read_bytes - 2] != b'\r' {
            panic!("invalid string");
        }

        String::from_utf8(buffer).expect("failed to convert to string")
    }

    pub fn new<'a>(stream: &'a mut BufReader<&'a mut T>) -> Reader<'a, T> {
        Reader { stream }
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::Reader;

    #[test]
    fn instantiation_works() {
        let mut s = "test\r\n".as_bytes();
        let mut b_reader = BufReader::new(&mut s);
        let _ = Reader::new(&mut b_reader);
    }

    #[test]
    fn parses_bulk_string() {
        let mut s = "$5\r\nHola!\r\n".as_bytes();
        let mut b_reader = BufReader::new(&mut s);
        let mut t = Reader::new(&mut b_reader);
        let result = t.read();
        assert_eq!(
            match result {
                crate::resp::Value::ValueBulk(bulk) => bulk.bulk == "Hola!",
                _ => false,
            },
            true
        )
    }

    #[test]
    fn parses_array() {
        let mut s = "*2\r\n$5\r\nhello\r\n$6\r\nworld!\r\n".as_bytes();
        let mut b_reader = BufReader::new(&mut s);
        let mut t = Reader::new(&mut b_reader);
        let result = t.read();
        assert_eq!(
            match result {
                crate::resp::Value::ValueArray(arr) => {
                    arr.arr.len() == 2
                }
                _ => false,
            },
            true
        )
    }
}
