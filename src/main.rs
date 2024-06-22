use std::collections::HashMap;

fn main() {
    // TODO add nil type
    // TODO implement simple INCR
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
        let command = str_command.to_command();

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
        }
    }
}

enum CommandType {
    Ping,
    Set,
    Get,
    Del,
    Exists,
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
        RespString {
            raw_str: format!("+{}\r\n", s),
            tokens: vec![s],
        }
    }

    pub fn bulk_from_string(s: String) -> Self {
        RespString {
            raw_str: format!("${}\r\n{}\r\n", s.len(), s),
            tokens: vec![s],
        }
    }

    pub fn integer_from_string(s: String) -> Self {
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
}
