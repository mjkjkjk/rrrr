use std::{
    io::{BufReader, Read, Write},
    net::TcpStream,
};

pub struct Client {
    connection: TcpStream,
}

impl Client {
    pub fn new(addr: &str) -> Self {
        Client {
            connection: TcpStream::connect(addr).expect("failed to connect"),
        }
    }

    pub fn write(&mut self, data: &str) {
        self.connection
            .write(data.as_bytes())
            .expect("failed to write");
    }

    pub fn read(mut self) -> String {
        let mut reader = BufReader::new(self.connection);
        let mut s = String::new();
        reader.read_to_string(&mut s);
        s
    }
}
