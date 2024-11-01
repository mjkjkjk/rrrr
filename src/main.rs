use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process::exit,
};

use command::Command;
use dotenv::dotenv;
use errors::ErrNum;
use log::debug;
use resp::{read_resp_from_stream, write_resp, RespValue};

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

fn handle_stream(mut stream: TcpStream, storage: Arc<Mutex<HashMap<String, String>>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let resp_value = match read_resp_from_stream(&mut reader) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                return;
            }
        };

        match resp_value.try_into() {
            Ok(command) => match command {
                Command::Ping => {
                    let response = RespValue::SimpleString("PONG".to_string());
                    if let Err(e) = write_resp(&response, &mut stream) {
                        eprintln!("Error writing response: {}", e);
                    }
                }
                Command::Get { key } => {
                    let storage = storage.lock().unwrap();
                    let response = match storage.get(&key) {
                        Some(value) => RespValue::BulkString(Some(value.clone())),
                        None => RespValue::BulkString(None),
                    };
                    if let Err(e) = write_resp(&response, &mut stream) {
                        eprintln!("Error writing response: {}", e);
                    }
                }
                Command::Set { key, value } => {
                    let mut storage = storage.lock().unwrap();
                    storage.insert(key, value);
                    let response = RespValue::SimpleString("OK".to_string());
                    if let Err(e) = write_resp(&response, &mut stream) {
                        eprintln!("Error writing response: {}", e);
                    }
                }
                Command::Del { keys } => println!("Got DEL command for keys: {:?}", keys),
                Command::CommandDocs => println!("Got COMMAND DOCS command"),
            },
            Err(e) => eprintln!("Error parsing command: {}", e),
        }
    }
}

fn main() {
    initialize_support_systems();

    let storage = Arc::new(Mutex::new(HashMap::new()));

    let server = initialize_server();

    for stream in server.incoming() {
        let storage = storage.clone();
        handle_stream(stream.unwrap(), storage);
    }
}
