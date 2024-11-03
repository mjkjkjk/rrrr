use std::collections::HashMap;
use std::convert::TryInto;
use std::io;
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
use resp::{read_resp_from_stream, write_resp, RespError, RespValue};

mod command;
mod errors;
mod resp;
mod util;

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

fn handle_command(command: Command, stream: &mut TcpStream, storage: &Arc<Mutex<HashMap<String, String>>>) {
    match command {
        Command::Ping => {
            let response = RespValue::SimpleString("PONG".to_string());
            if let Err(e) = write_resp(&response, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
        Command::Get { key } => {
            let storage = storage.lock().unwrap();
            let response = match storage.get(&key) {
                Some(value) => RespValue::BulkString(Some(value.clone())),
                None => RespValue::BulkString(None),
            };
            if let Err(e) = write_resp(&response, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
        Command::Set { key, value } => {
            let mut storage = storage.lock().unwrap();
            storage.insert(key, value);
            let response = RespValue::SimpleString("OK".to_string());
            if let Err(e) = write_resp(&response, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
        Command::Del { keys } => println!("Got DEL command for keys: {:?}", keys),
        Command::CommandDocs => println!("Got COMMAND DOCS command"),
        Command::IncrBy { key, value } => {
            let mut storage = storage.lock().unwrap();
            let increment = match value.parse::<i64>() {
                Ok(n) => n,
                Err(_) => {
                    let response = RespValue::Error(
                        "ERR value is not an integer or out of range".to_string(),
                    );
                    write_resp(&response, stream);
                    return;
                }
            };
            if let Err(e) = handle_numeric_operation(&mut storage, key, |n| n + increment, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
        Command::Incr { key } => {
            let mut storage = storage.lock().unwrap();
            if let Err(e) = handle_numeric_operation(&mut storage, key, |n| n + 1, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
        Command::DecrBy { key, value } => {
            let mut storage = storage.lock().unwrap();
            let decrement = match value.parse::<i64>() {
                Ok(n) => n,
                Err(_) => {
                    let response = RespValue::Error(
                        "ERR value is not an integer or out of range".to_string(),
                    );
                    write_resp(&response, stream);
                    return;
                }
            };
            if let Err(e) = handle_numeric_operation(&mut storage, key, |n| n - decrement, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
        Command::Decr { key } => {
            let mut storage = storage.lock().unwrap();
            if let Err(e) = handle_numeric_operation(&mut storage, key, |n| n - 1, stream) {
                eprintln!("Error writing response: {}", e);
                return;
            }
        }
    }
}

fn handle_numeric_operation(
    storage: &mut std::sync::MutexGuard<HashMap<String, String>>,
    key: String,
    operation: impl FnOnce(i64) -> i64,
    stream: &mut TcpStream,
) -> Result<(), io::Error> {
    let default = "0".to_string();
    let current_value = storage.get(&key).unwrap_or(&default);

    let current_num = match current_value.parse::<i64>() {
        Ok(n) => n,
        Err(_) => {
            let response = RespValue::Error(
                "ERR value is not an integer or out of range".to_string(),
            );
            write_resp(&response, stream)?;
            return Ok(());
        }
    };

    let new_value = operation(current_num);
    storage.insert(key, new_value.to_string());

    let response = RespValue::Integer(new_value);
    write_resp(&response, stream)?;
    Ok(())
}

fn handle_stream(mut stream: TcpStream, storage: Arc<Mutex<HashMap<String, String>>>) {
    stream.set_nonblocking(false).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let resp_value = match read_resp_from_stream(&mut reader) {
            Ok(value) => value,
            Err(e) => {
                if let RespError::IoError(io_err) = &e {
                    if io_err.kind() == io::ErrorKind::UnexpectedEof || 
                       io_err.kind() == io::ErrorKind::ConnectionReset {
                        return;
                    }
                }
                eprintln!("Error reading from stream: {}", e);
                continue;
            }
        };

        match resp_value.try_into() {
            Ok(command) => handle_command(command, &mut stream, &storage),
            Err(e) => {
                eprintln!("Error parsing command: {}", e);
                let response = RespValue::Error(e.to_string());
                if let Err(e) = write_resp(&response, &mut stream) {
                    eprintln!("Error writing response: {}", e);
                    break;
                }
                continue;
            }
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
