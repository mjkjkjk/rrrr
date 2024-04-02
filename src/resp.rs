use std::io::{BufRead, BufReader, Read};

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
            _ => panic!("unknown type"),
        }
    }

    fn read_array(&mut self) -> Value {
        let len = self.read_integer();

        let mut v: Vec<Value> = vec![];

        for _i in 0..len {
            let val = self.read();
            v.push(val);
        }

        Value {
            typ: "array".to_string(),
            str: "".to_string(),
            num: len,
            bulk: "".to_string(),
            arr: v,
        }
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

        Value {
            typ: "bulk".to_string(),
            str: "".to_string(),
            num: 0,
            bulk: read_string.to_string(),
            arr: vec![],
        }
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
        assert_eq!(result.bulk, "Hola!");
    }

    #[test]
    fn parses_array() {
        let mut s = "*2\r\n$5\r\nhello\r\n$6\r\nworld!\r\n".as_bytes();
        let mut b_reader = BufReader::new(&mut s);
        let mut t = Reader::new(&mut b_reader);
        let result = t.read();
        assert_eq!(result.arr.len(), 2);
        assert_eq!(result.arr[0].bulk, "hello");
        assert_eq!(result.arr[1].bulk, "world!");
    }
}
