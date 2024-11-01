use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, PartialEq, Clone)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),    // None represents Null bulk string
    Array(Option<Vec<RespValue>>), // None represents Null array
}

#[derive(Debug)]
pub enum RespError {
    IoError(io::Error),
    ParseError(String),
    InvalidLength,
    InvalidUtf8,
}

impl From<io::Error> for RespError {
    fn from(error: io::Error) -> Self {
        RespError::IoError(error)
    }
}

impl std::fmt::Display for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RespError::IoError(e) => write!(f, "IO error: {}", e),
            RespError::ParseError(s) => write!(f, "Parse error: {}", s),
            RespError::InvalidLength => write!(f, "Invalid length"),
            RespError::InvalidUtf8 => write!(f, "Invalid UTF-8"),
        }
    }
}

impl std::error::Error for RespError {}

pub fn read_resp<R: BufRead>(reader: &mut R) -> Result<RespValue, RespError> {
    let mut first_byte = [0u8; 1];
    reader.read_exact(&mut first_byte)?;

    match first_byte[0] as char {
        '+' => read_simple_string(reader),
        '-' => read_error(reader),
        ':' => read_integer(reader),
        '$' => read_bulk_string(reader),
        '*' => read_array(reader),
        _ => Err(RespError::ParseError(format!(
            "Invalid RESP type byte: {}",
            first_byte[0] as char
        ))),
    }
}

fn read_line<R: BufRead>(reader: &mut R) -> Result<String, RespError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim_end_matches("\r\n").to_string())
}

fn read_simple_string<R: BufRead>(reader: &mut R) -> Result<RespValue, RespError> {
    Ok(RespValue::SimpleString(read_line(reader)?))
}

fn read_error<R: BufRead>(reader: &mut R) -> Result<RespValue, RespError> {
    Ok(RespValue::Error(read_line(reader)?))
}

fn read_integer<R: BufRead>(reader: &mut R) -> Result<RespValue, RespError> {
    let line = read_line(reader)?;
    let num = line
        .parse::<i64>()
        .map_err(|_| RespError::ParseError("Invalid integer".to_string()))?;
    Ok(RespValue::Integer(num))
}

fn read_bulk_string<R: BufRead>(reader: &mut R) -> Result<RespValue, RespError> {
    let length_str = read_line(reader)?;
    let length = length_str
        .parse::<i64>()
        .map_err(|_| RespError::ParseError("Invalid bulk string length".to_string()))?;

    if length == -1 {
        return Ok(RespValue::BulkString(None));
    }

    if length < 0 {
        return Err(RespError::ParseError(
            "Invalid bulk string length".to_string(),
        ));
    }

    let length = length as usize;
    let mut buf = vec![0u8; length + 2]; // +2 for CRLF
    reader.read_exact(&mut buf)?;

    if buf[length..] != b"\r\n"[..] {
        return Err(RespError::ParseError("Missing CRLF".to_string()));
    }

    let s = String::from_utf8(buf[..length].to_vec()).map_err(|_| RespError::InvalidUtf8)?;

    Ok(RespValue::BulkString(Some(s)))
}

fn read_array<R: BufRead>(reader: &mut R) -> Result<RespValue, RespError> {
    let length_str = read_line(reader)?;
    let length = length_str
        .parse::<i64>()
        .map_err(|_| RespError::ParseError("Invalid array length".to_string()))?;

    if length == -1 {
        return Ok(RespValue::Array(None));
    }

    if length < 0 {
        return Err(RespError::ParseError("Invalid array length".to_string()));
    }

    let length = length as usize;
    let mut values = Vec::with_capacity(length);

    for _ in 0..length {
        values.push(read_resp(reader)?);
    }

    Ok(RespValue::Array(Some(values)))
}

// Helper function specifically for TcpStream that sets timeout
pub fn read_resp_from_stream(
    stream: &mut io::BufReader<TcpStream>,
) -> Result<RespValue, RespError> {
    stream
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(5)))?;
    read_resp(stream)
}

pub fn write_resp(value: &RespValue, writer: &mut impl Write) -> io::Result<()> {
    match value {
        RespValue::SimpleString(s) => {
            writer.write_all(b"+")?;
            writer.write_all(s.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        RespValue::Error(msg) => {
            writer.write_all(b"-")?;
            writer.write_all(msg.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        RespValue::Integer(n) => {
            writer.write_all(b":")?;
            writer.write_all(n.to_string().as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        RespValue::BulkString(opt) => {
            writer.write_all(b"$")?;
            match opt {
                None => writer.write_all(b"-1\r\n")?,
                Some(s) => {
                    writer.write_all(s.len().to_string().as_bytes())?;
                    writer.write_all(b"\r\n")?;
                    writer.write_all(s.as_bytes())?;
                    writer.write_all(b"\r\n")?;
                }
            }
        }
        RespValue::Array(opt) => {
            writer.write_all(b"*")?;
            match opt {
                None => writer.write_all(b"-1\r\n")?,
                Some(arr) => {
                    writer.write_all(arr.len().to_string().as_bytes())?;
                    writer.write_all(b"\r\n")?;
                    for item in arr {
                        write_resp(item, writer)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_simple_string() {
        let input = "+OK\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(
            read_resp(&mut reader).unwrap(),
            RespValue::SimpleString("OK".to_string())
        );
    }

    #[test]
    fn test_error() {
        let input = "-Error message\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(
            read_resp(&mut reader).unwrap(),
            RespValue::Error("Error message".to_string())
        );
    }

    #[test]
    fn test_integer() {
        let input = ":1234\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(read_resp(&mut reader).unwrap(), RespValue::Integer(1234));
    }

    #[test]
    fn test_bulk_string() {
        let input = "$6\r\nfoobar\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(
            read_resp(&mut reader).unwrap(),
            RespValue::BulkString(Some("foobar".to_string()))
        );
    }

    #[test]
    fn test_null_bulk_string() {
        let input = "$-1\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(read_resp(&mut reader).unwrap(), RespValue::BulkString(None));
    }

    #[test]
    fn test_array() {
        let input = "*2\r\n$3\r\nGET\r\n$4\r\nkeys\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(
            read_resp(&mut reader).unwrap(),
            RespValue::Array(Some(vec![
                RespValue::BulkString(Some("GET".to_string())),
                RespValue::BulkString(Some("keys".to_string())),
            ]))
        );
    }

    #[test]
    fn test_null_array() {
        let input = "*-1\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(read_resp(&mut reader).unwrap(), RespValue::Array(None));
    }

    #[test]
    fn test_nested_array() {
        let input = "*2\r\n*2\r\n+OK\r\n:1234\r\n$6\r\nfoobar\r\n";
        let mut reader = io::BufReader::new(Cursor::new(input));
        assert_eq!(
            read_resp(&mut reader).unwrap(),
            RespValue::Array(Some(vec![
                RespValue::Array(Some(vec![
                    RespValue::SimpleString("OK".to_string()),
                    RespValue::Integer(1234),
                ])),
                RespValue::BulkString(Some("foobar".to_string())),
            ]))
        );
    }
}
