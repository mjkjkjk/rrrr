use std::{
    collections::HashMap,
    io::Read,
    net::{TcpListener, TcpStream},
};

use glob::Pattern;

use dotenv::dotenv;
use log::{debug, info};
use resp_handler::RespHandler;
use resp_string::RespString;

mod resp_handler;
mod resp_string;

fn main() {
    dotenv().ok();
    env_logger::init();

    // TODO add nil type
    // TODO implement simple INCR
    // TODO simple server, responds to string commands
    // TODO add KEYS command
    // TODO refactor simple strings like OK for tests

    let listener = TcpListener::bind("127.0.0.1:6379").expect("could not bind address");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = String::new();
                let _ = stream.read_to_string(&mut buf);
                let command = RespString::from_string(buf);
                let mut handler = RespHandler::new();
                let result = handler.handle(command);
                println!("{}", result.to_string());
            }
            Err(_) => todo!(),
        }
    }
}

#[derive(Debug)]
enum CommandType {
    Ping,
    Set,
    Get,
    Del,
    Exists,
    Keys,
    Incr,
}

pub struct Command {
    kind: CommandType,
    tokens: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::{RespHandler, RespString};

    #[test]
    fn convert_simple_ping() {
        let raw_command = "PING".to_string();
        let expected = "*1\r\n$4\r\nPING\r\n";
        assert_eq!(
            RespString::from_string(raw_command).to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn convert_ping_with_one_argument() {
        let raw_command = "PING \"test\"".to_string();
        let expected = "*2\r\n$4\r\nPING\r\n$6\r\n\"test\"\r\n";
        assert_eq!(
            RespString::from_string(raw_command).to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn convert_ping_with_multiple_arguments() {
        let raw_command = "PING \"test\" \"another\"".to_string();
        let expected = "*3\r\n$4\r\nPING\r\n$6\r\n\"test\"\r\n$9\r\n\"another\"\r\n";
        assert_eq!(
            RespString::from_string(raw_command).to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn handle_ping_without_arguments() {
        let command = RespString::from_string("PING".to_string());
        let mut handler = RespHandler::new();
        let result = handler.handle(command);
        let expected = "+PONG\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_ping_with_argument() {
        let command = RespString::from_string("PING \"test\"".to_string());
        let mut handler = RespHandler::new();
        let result = handler.handle(command);
        let expected = "$6\r\n\"test\"\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_set_with_simple_value() {
        let command = RespString::from_string("SET test_key test_value".to_string());
        let mut handler = RespHandler::new();
        let result = handler.handle(command);
        let expected = "+OK\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_get_with_undefined_key() {
        let mut handler = RespHandler::new();
        let get_command = RespString::from_string("GET test_key".to_string());
        let get_result = handler.handle(get_command);
        let get_expected = "+(nil)\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);
    }

    #[test]
    fn handle_set_with_simple_value_get_to_retrieve() {
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let get_command = RespString::from_string("GET test_key".to_string());
        let get_result = handler.handle(get_command);
        let get_expected = "+test_value\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);
    }

    #[test]
    fn handle_double_set_simple_value() {
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let get_command = RespString::from_string("GET test_key".to_string());
        let get_result = handler.handle(get_command);
        let get_expected = "+test_value\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);

        let set_command = RespString::from_string("SET test_key test_value_2".to_string());
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let get_command = RespString::from_string("GET test_key".to_string());
        let get_result = handler.handle(get_command);
        let get_expected = "+test_value_2\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);
    }

    #[test]
    fn handle_delete_undefined_key() {
        let mut handler = RespHandler::new();
        let command = RespString::from_string("DEL test_key".to_string());
        let result = handler.handle(command);
        let expected = ":0\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_delete_one_key() {
        let mut handler = RespHandler::new();
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        handler.handle(set_command);
        let del_command = RespString::from_string("DEL test_key".to_string());
        let del_result = handler.handle(del_command);
        let del_expected = ":1\r\n".to_string();
        assert_eq!(del_result.to_string(), del_expected);
    }

    #[test]
    fn handle_delete_multiple_keys() {
        let mut handler = RespHandler::new();
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        handler.handle(set_command);
        let set_command = RespString::from_string("SET test_key2 test_value2".to_string());
        handler.handle(set_command);
        let set_command = RespString::from_string("SET test_key3 test_value3".to_string());
        handler.handle(set_command);
        let del_command = RespString::from_string("DEL test_key test_key2 test_key3".to_string());
        let del_result = handler.handle(del_command);
        let del_expected = ":3\r\n".to_string();
        assert_eq!(del_result.to_string(), del_expected);
    }

    #[test]
    fn handle_exists_undefined_key() {
        let mut handler = RespHandler::new();
        let command = RespString::from_string("EXISTS test_key".to_string());
        let result = handler.handle(command);
        let expected = ":0\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_exists_all_defined_keys() {
        let mut handler = RespHandler::new();
        let command = RespString::from_string("SET k1 v1".to_string());
        handler.handle(command);
        let command = RespString::from_string("SET k2 v1".to_string());
        handler.handle(command);
        let command = RespString::from_string("SET k3 v1".to_string());
        handler.handle(command);
        let command = RespString::from_string("EXISTS k1 k2 k3".to_string());
        let result = handler.handle(command);
        let expected = ":3\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_exists_some_defined_keys() {
        let mut handler = RespHandler::new();
        let command = RespString::from_string("SET k1 v1".to_string());
        handler.handle(command);
        let command = RespString::from_string("SET k2 v1".to_string());
        handler.handle(command);
        let command = RespString::from_string("SET k3 v1".to_string());
        handler.handle(command);
        let command = RespString::from_string("EXISTS k1 k2 k3 k4".to_string());
        let result = handler.handle(command);
        let expected = ":3\r\n".to_string();
        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_subkey_set() {
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let get_command = RespString::from_string("GET test_key".to_string());
        let get_result = handler.handle(get_command);
        let get_expected = "+test_value\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);

        let set_command2 = RespString::from_string("SET test_key:sub test_value2".to_string());
        let set_result = handler.handle(set_command2);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let get_command2 = RespString::from_string("GET test_key:sub".to_string());
        let get_result = handler.handle(get_command2);
        let get_expected = "+test_value2\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);
    }

    #[test]
    fn handle_get_with_multiple_keys() {
        /* TODO */
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let get_command = RespString::from_string("GET test_key different_key".to_string());
        let get_result = handler.handle(get_command);
        let get_expected = "+(error) ERR wrong number of arguments for command\r\n".to_string();
        assert_eq!(get_result.to_string(), get_expected);

        panic!("TODO");
    }

    #[test]
    fn handle_set_with_multiple_keys() {
        let set_command = RespString::from_string(
            "SET test_key test_value another_key another_value".to_string(),
        );
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+(error) syntax error\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);
    }

    #[test]
    fn handle_keys_undefined_key() {
        todo!()
    }

    #[test]
    fn handle_keys_single_key() {
        let set_command = RespString::from_string("SET test_key test_value".to_string());
        let mut handler = RespHandler::new();
        let set_result = handler.handle(set_command);
        let set_expected = "+OK\r\n".to_string();
        assert_eq!(set_result.to_string(), set_expected);

        let keys_command = RespString::from_string("KEYS test_key".to_string());
        let keys_result = handler.handle(keys_command);
        let keys_expected = "*1\r\n+test_key\r\n"; // TODO check if simple or bulk string reply is more correct
        assert_eq!(keys_result.to_string(), keys_expected);
    }

    #[test]
    fn handle_keys_multiple_keys() {
        let mut handler = RespHandler::new();

        let keys_command = RespString::from_string("KEYS key1 key2 key3".to_string());
        let keys_result = handler.handle(keys_command);
        let keys_expected = "+(error) ERR wrong number of arguments for command\r\n";

        assert_eq!(keys_result.to_string(), keys_expected);
    }

    #[test]
    fn handle_keys_multiple_arguments() {
        todo!()
    }

    #[test]
    fn handle_incr_undefined() {
        let mut handler = RespHandler::new();

        let command = RespString::from_string("INCR key1".to_string());
        let result = handler.handle(command);
        let expected = ":1\r\n";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_incr_integer() {
        let mut handler = RespHandler::new();

        let command = RespString::from_string("INCR key1".to_string());
        let result = handler.handle(command);
        let expected = ":1\r\n";

        assert_eq!(result.to_string(), expected);

        let command = RespString::from_string("INCR key1".to_string());
        let result = handler.handle(command);
        let expected = ":2\r\n";

        assert_eq!(result.to_string(), expected);
    }

    #[test]
    fn handle_incr_invalid() {
        let mut handler = RespHandler::new();

        let command = RespString::from_string("SET key1 test".to_string());
        let result = handler.handle(command);
        let expected = "+OK\r\n";

        assert_eq!(result.to_string(), expected);

        let command = RespString::from_string("INCR key1".to_string());
        let result = handler.handle(command);
        let expected = "+(error) value is not an integer or out of range\r\n";

        assert_eq!(result.to_string(), expected);
    }
}
