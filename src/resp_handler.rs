use std::collections::HashMap;

use glob::Pattern;
use log::info;

use crate::{resp_string::RespString, CommandType};

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
            CommandType::Incr => {
                if command.tokens.len() > 2 {
                    return RespString::simple_from_string(
                        "(error) ERR wrong number of arguments for command".to_string(),
                    );
                }

                let key = &command.tokens[1];

                if !self.data.contains_key(key) {
                    self.data.insert(key.to_string(), "1".to_string()); // TODO save as integer?
                    return RespString::integer_from_string("1".to_string());
                }

                let value = self
                    .data
                    .get(key)
                    .expect("error loading key")
                    .parse::<i64>();

                match value {
                    Ok(v) => {
                        self.data.insert(key.to_string(), (v + 1).to_string());
                        return RespString::integer_from_string((v + 1).to_string());
                    }
                    Err(_) => RespString::simple_from_string(
                        "(error) value is not an integer or out of range".to_string(),
                    ),
                }
            }
        }
    }
}
