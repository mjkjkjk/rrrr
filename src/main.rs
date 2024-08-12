use std::{
    collections::HashMap,
    io::Read,
    net::{TcpListener, TcpStream},
};

use glob::Pattern;

use dotenv::dotenv;
use log::{debug, info};

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

pub struct RespHandler {
    data: HashMap<String, String>,
}

impl RespHandler {
    pub fn new() -> Self {
        RespHandler {
            data: HashMap::new(),
        }
    }

    pub fn handle(&mut self, str_command: RespString) -> RespString {
        info!("Handling command");
        let command = str_command.to_command();

        info!("Command kind: {:?}", command.kind);

        match command.kind {
            CommandType::Get => {
                if command.tokens.len() != 2 {
                    return RespString::simple_from_string(
                        "(error) ERR wrong number of arguments for command".to_string(),
                    );
                }

                let key = &command.tokens[1];
                let value = self.data.get(key);
                match value {
                    Some(value) => RespString::simple_from_string(value.to_string()),
                    None => RespString::simple_from_string("(nil)".to_string()),
                }
            }
            CommandType::Set => {
                if command.tokens.len() != 3 {
                    return RespString::simple_from_string(
                        "(error) ERR wrong number of arguments for command".to_string(),
                    );
                }

                let key = &command.tokens[1];
                let value = &command.tokens[2];

                self.data.insert(key.to_string(), value.to_string());

                return RespString::simple_from_string("OK".to_string());
            }
            CommandType::Ping => {
                if command.tokens.len() == 1 {
                    return RespString::simple_from_string("PONG".to_string());
                }

                if command.tokens.len() == 2 {
                    let arg = &command.tokens[1];
                    return RespString::bulk_from_string(arg.to_string());
                }

                panic!("can't handle ping with more than 1 argument")
            }
            CommandType::Del => {
                let mut deleted = 0;
                command.tokens[1..].iter().for_each(|key| {
                    if self.data.contains_key(key) {
                        self.data.remove(key);
                        deleted += 1;
                    }
                });

                RespString::integer_from_string(format!("{}", deleted))
            }
            CommandType::Exists => {
                let count = command.tokens[1..].iter().fold(0, |acc, key| {
                    acc + (if self.data.contains_key(key) { 1 } else { 0 })
                });

                RespString::integer_from_string(format!("{}", count))
            }
            CommandType::Keys => {
                if command.tokens.len() != 2 {
                    return RespString::simple_from_string(
                        "(error) ERR wrong number of arguments for command".to_string(),
                    );
                }

                let glob = Pattern::new(command.tokens[1].as_str()).unwrap();
                // TODO do without clone
                let matched_keys = <HashMap<String, String> as Clone>::clone(&self.data)
                    .into_iter()
                    .filter(|(key, _)| glob.matches(key.as_str()))
                    .map(|(k, _)| k);

                RespString::strings_to_array(matched_keys.collect::<Vec<String>>())
            }
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
}

pub struct Command {
    kind: CommandType,
    tokens: Vec<String>,
}

pub struct RespString {
    raw_str: String,
    tokens: Vec<String>,
}

impl RespString {
    pub fn from_string(s: String) -> Self {
        let parts = s.split(" ");
        let collected = parts.collect::<Vec<&str>>();
        let prefix = format!("*{}\r\n", collected.len());
        let command_parts = collected.iter();
        let content = command_parts
            .clone()
            .map(|part| format!("${}\r\n{}\r\n", part.len(), part))
            .fold("".to_string(), |acc, x| acc + &x);

        let tokens = command_parts
            .map(|e| e.to_string())
            .collect::<Vec<String>>();

        RespString {
            raw_str: prefix + &content,
            tokens,
        }
    }

    pub fn simple_from_string(s: String) -> Self {
        debug!("simple string");
        RespString {
            raw_str: format!("+{}\r\n", s),
            tokens: vec![s],
        }
    }

    pub fn strings_to_array(mut s: Vec<String>) -> Self {
        debug!("array of simple strings");
        let len = s.len();
        let joined = s
            .iter()
            .map(|simple| Self::simple_from_string(simple.to_string()).to_string())
            .collect::<Vec<String>>()
            .join("\r\n");
        s.push("\r\n".to_string());
        RespString {
            raw_str: format!("*{}\r\n{}", len, joined),
            tokens: s,
        }
    }

    pub fn bulk_from_string(s: String) -> Self {
        debug!("bulk string");
        RespString {
            raw_str: format!("${}\r\n{}\r\n", s.len(), s),
            tokens: vec![s],
        }
    }

    pub fn integer_from_string(s: String) -> Self {
        debug!("integer");
        let num = str::parse::<i64>(&s);
        match num {
            Ok(num) => RespString {
                raw_str: format!(":{}\r\n", num),
                tokens: vec![s],
            },
            Err(_) => todo!(),
        }
    }

    pub fn to_command(self) -> Command {
        let typ = self
            .tokens
            .first()
            .expect("can't have empty commands")
            .as_str();

        debug!("mapping to command, type: {}", typ);

        match typ {
            "SET" => Command {
                kind: CommandType::Set,
                tokens: self.tokens,
            },
            "GET" => Command {
                kind: CommandType::Get,
                tokens: self.tokens,
            },
            "PING" => Command {
                kind: CommandType::Ping,
                tokens: self.tokens,
            },
            "DEL" => Command {
                kind: CommandType::Del,
                tokens: self.tokens,
            },
            "EXISTS" => Command {
                kind: CommandType::Exists,
                tokens: self.tokens,
            },
            "KEYS" => Command {
                kind: CommandType::Keys,
                tokens: self.tokens,
            },
            _ => panic!("not implemented"),
        }
    }
}

impl ToString for RespString {
    fn to_string(&self) -> String {
        self.raw_str.to_owned()
    }
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
        panic!("TODO");
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
        panic!("TODO");
    }

    fn handle_keys_multiple_arguments() {
        panic!("TODO");
    }
}
