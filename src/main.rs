use std::convert::TryInto;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process::exit,
};

use command::Command;
use dotenv::dotenv;
use errors::ErrNum;
use log::debug;
use resp::read_resp_from_stream;

mod command;
mod errors;
mod resp;

fn initialize_support_systems() {
    match dotenv() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to initialize dotenv: {:?}", e);
            std::process::exit(ErrNum::Configuration as i32);
        }
    }
    env_logger::init();
}

fn initialize_server() -> TcpListener {
    let listener = match TcpListener::bind("127.0.0.1:6379") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to initialize TcpListener: {:?}", e);
            std::process::exit(e.raw_os_error().unwrap_or(ErrNum::Connection as i32));
        }
    };

    listener
}

fn handle_stream(stream: TcpStream) {
    let mut reader = BufReader::new(stream);

    let resp_value = read_resp_from_stream(&mut reader).unwrap();

    match resp_value.try_into() {
        Ok(command) => match command {
            Command::Get { key } => println!("Got GET command for key: {}", key),
            Command::Set { key, value } => {
                println!("Got SET command for key: {} with value: {}", key, value)
            }
            Command::Del { keys } => println!("Got DEL command for keys: {:?}", keys),
            Command::Ping => println!("Got PING command"),
        },
        Err(e) => eprintln!("Error parsing command: {}", e),
    }
}

fn main() {
    initialize_support_systems();

    let server = initialize_server();

    for stream in server.incoming() {
        handle_stream(stream.unwrap());
    }
}
