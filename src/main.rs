mod resp;
pub mod handler;

use std::{
    io::{BufReader, BufWriter, Write},
    net::TcpListener,
};

use resp::ValueString;

use crate::resp::Reader;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6379").expect("failed to bind");

    for stream in listener.incoming() {
        let mut s = stream.expect("failed to open stream");
        // let result = handle_client(&s);
        // println!("{}", result); // TODO result has no \0 at the end???
        // s.write(result.as_bytes()).expect("failed to write back");

        {
            let mut b_reader = BufReader::new(&mut s);
            let mut reader = Reader::new(&mut b_reader);
            let res = reader.read();
            println!("{:?}", res);
            println!("{:?}", res.marshall());
        }

        {
            let mut b_writer = BufWriter::new(&mut s);

            b_writer.write(
                &resp::Value::ValueString(ValueString {
                    str: "OK".to_string(),
                })
                .marshall(),
            );
            b_writer.flush();
        }

        // let _ = s.write("+OK\r\n".as_bytes());
    }
}
