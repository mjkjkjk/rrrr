pub mod handler;
mod resp;

use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpListener,
};

use resp::{ValueString, Writer};

use crate::resp::Reader;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6379").expect("failed to bind");

    for stream in listener.incoming() {
        let mut s = stream.expect("failed to open stream");

        {
            let mut b_reader = BufReader::new(&mut s);
            let mut reader = Reader::new(&mut b_reader);
            let res = reader.read();

            let mut b_writer = BufWriter::new(&mut s);
            let mut writer = Writer::new(b_writer);
            writer.write(res);
        }
    }
}
