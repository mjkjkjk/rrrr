use std::convert::TryInto;
use std::fs::File;
use std::io::{self, BufWriter};
use std::sync::{Arc, Mutex};
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

use command::Command;
use command_handler::handle_command;
use dotenvy::dotenv;
use errors::ErrNum;
use log::debug;
use resp::{read_resp_from_stream, write_resp, RespError, RespValue};
use storage::Storage;

mod command;
mod errors;
mod resp;
mod storage;
mod util;

mod command_handler;
mod logger;
use logger::Logger;

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

fn handle_file(mut file: File, storage: Arc<Mutex<Storage>>) {
    let mut reader = BufReader::new(file);
    loop {
        let resp_value = read_resp_from_stream(&mut reader).unwrap();

        if let RespValue::Array(Some(command_array)) = &resp_value {
            let _ = match resp_value.try_into() {
                Ok(command) => handle_command(command, &storage),
                Err(e) => RespValue::Error(e.to_string()),
            };
        }
    }
}

fn handle_stream(mut stream: TcpStream, storage: Arc<Mutex<Storage>>, logger: Arc<Logger>) {
    stream.set_nonblocking(false).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    loop {
        let resp_value = match read_resp_from_stream(&mut reader) {
            Ok(value) => value,
            Err(e) => {
                if let RespError::IoError(io_err) = &e {
                    if io_err.kind() == io::ErrorKind::UnexpectedEof
                        || io_err.kind() == io::ErrorKind::ConnectionReset
                    {
                        return;
                    }
                }
                eprintln!("Error reading from stream: {}", e);
                continue;
            }
        };

        if let RespValue::Array(Some(command_array)) = &resp_value {
            if let Some(RespValue::BulkString(Some(cmd_name))) = command_array.first() {
                let command_str = command_array
                    .iter()
                    .skip(1)
                    .map(|v| match v {
                        RespValue::BulkString(Some(s)) => s.to_string(),
                        RespValue::SimpleString(s) => s.to_string(),
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                logger.log(format!("{} {}", cmd_name.to_uppercase(), command_str));
            }

            let response = match resp_value.try_into() {
                Ok(command) => handle_command(command, &storage),
                Err(e) => RespValue::Error(e.to_string()),
            };
            let mut writer = BufWriter::new(&mut stream);
            if let Err(e) = write_resp(&response, &mut writer) {
                eprintln!("Error writing response: {}", e);
                break;
            }
        } else {
            let response = RespValue::Error("Invalid command".to_string());
            let mut writer = BufWriter::new(&mut stream);
            if let Err(e) = write_resp(&response, &mut writer) {
                eprintln!("Error writing response: {}", e);
                break;
            }
        }
    }
}

fn main() {
    initialize_support_systems();

    let storage = Arc::new(Mutex::new(Storage::new()));
    let log_file = std::env::var("COMMAND_LOG").unwrap_or_else(|_| "commands.log".to_string());
    let logger = Arc::new(Logger::new(log_file));

    let server = initialize_server();

    for stream in server.incoming() {
        let storage = storage.clone();
        let logger = logger.clone();
        let file = File::open("commands.log").unwrap();
        //handle_file(file, storage.clone());
        handle_stream(stream.unwrap(), storage.clone(), logger);
    }
}
