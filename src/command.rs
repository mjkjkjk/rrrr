use crate::resp::RespValue;
use std::string::ToString;

#[derive(Debug, PartialEq)]
pub enum Command {
    Get { key: String },
    MGet { keys: Vec<String> },
    Set { key: String, value: String },
    Del { keys: Vec<String> },
    IncrBy { key: String, value: String },
    Incr { key: String },
    DecrBy { key: String, value: String },
    Decr { key: String },
    Exists { keys: Vec<String> },
    Expire { key: String, expire: String },
    TTL { key: String },
    Ping,
    CommandDocs,
    FlushAll,
}

#[derive(Debug)]
pub enum CommandError {
    WrongNumberOfArguments {
        cmd: String,
        expected: usize,
        got: usize,
    },
    ParseError(String),
    UnknownCommand(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::WrongNumberOfArguments { cmd, expected, got } => {
                write!(
                    f,
                    "wrong number of arguments for '{}' command: expected {}, got {}",
                    cmd, expected, got
                )
            }
            CommandError::ParseError(msg) => write!(f, "parse error: {}", msg),
            CommandError::UnknownCommand(cmd) => write!(f, "unknown command '{}'", cmd),
        }
    }
}

impl std::error::Error for CommandError {}

impl TryFrom<RespValue> for Command {
    type Error = CommandError;

    fn try_from(value: RespValue) -> Result<Self, Self::Error> {
        match value {
            RespValue::Array(Some(array)) => {
                if array.is_empty() {
                    return Err(CommandError::ParseError("empty command".to_string()));
                }

                // Get the command name from the first argument
                let command_name = match &array[0] {
                    RespValue::BulkString(Some(s)) => s.to_uppercase(),
                    _ => {
                        return Err(CommandError::ParseError(
                            "command name must be a bulk string".to_string(),
                        ))
                    }
                };

                match command_name.as_str() {
                    "GET" => {
                        if array.len() != 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "GET".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }

                        let key = extract_string(&array[1])?;
                        Ok(Command::Get { key })
                    }

                    "MGET" => {
                        if array.len() < 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "MGET".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }
                        let keys = array[1..]
                            .iter()
                            .map(|v| extract_string(v))
                            .collect::<Result<Vec<String>, _>>()?;
                        Ok(Command::MGet { keys })
                    }

                    "SET" => {
                        if array.len() != 3 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "SET".to_string(),
                                expected: 3,
                                got: array.len(),
                            });
                        }

                        let key = extract_string(&array[1])?;
                        let value = extract_string(&array[2])?;
                        Ok(Command::Set { key, value })
                    }

                    "INCRBY" => {
                        if array.len() != 3 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "INCRBY".to_string(),
                                expected: 3,
                                got: array.len(),
                            });
                        }

                        let key = extract_string(&array[1])?;
                        let value = extract_string(&array[2])?;
                        Ok(Command::IncrBy { key, value })
                    }

                    "INCR" => {
                        if array.len() != 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "INCR".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }
                        let key = extract_string(&array[1])?;
                        Ok(Command::Incr { key })
                    }

                    "DECRBY" => {
                        if array.len() != 3 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "DECRBY".to_string(),
                                expected: 3,
                                got: array.len(),
                            });
                        }

                        let key = extract_string(&array[1])?;
                        let value = extract_string(&array[2])?;
                        Ok(Command::DecrBy { key, value })
                    }

                    "DECR" => {
                        if array.len() != 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "DECR".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }
                        let key = extract_string(&array[1])?;
                        Ok(Command::Decr { key })
                    }

                    "DEL" => {
                        if array.len() < 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "DEL".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }

                        let mut keys = Vec::with_capacity(array.len() - 1);
                        for arg in &array[1..] {
                            keys.push(extract_string(arg)?);
                        }
                        Ok(Command::Del { keys })
                    }

                    "PING" => {
                        if array.len() != 1 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "PING".to_string(),
                                expected: 1,
                                got: array.len(),
                            });
                        }
                        Ok(Command::Ping)
                    }

                    "COMMAND" => {
                        if array.len() != 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "COMMAND".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }

                        Ok(Command::CommandDocs)
                    }

                    "EXISTS" => {
                        if array.len() < 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "EXISTS".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }

                        let keys = array[1..]
                            .iter()
                            .map(|v| extract_string(v))
                            .collect::<Result<Vec<String>, _>>()?;
                        Ok(Command::Exists { keys })
                    }

                    "EXPIRE" => {
                        if array.len() != 3 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "EXPIRE".to_string(),
                                expected: 3,
                                got: array.len(),
                            });
                        }

                        let key = extract_string(&array[1])?;
                        let expire = extract_string(&array[2])?;
                        Ok(Command::Expire { key, expire })
                    }

                    "TTL" => {
                        if array.len() != 2 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "TTL".to_string(),
                                expected: 2,
                                got: array.len(),
                            });
                        }
                        let key = extract_string(&array[1])?;
                        Ok(Command::TTL { key })
                    }

                    "FLUSHALL" => {
                        if array.len() != 1 {
                            return Err(CommandError::WrongNumberOfArguments {
                                cmd: "FLUSHALL".to_string(),
                                expected: 1,
                                got: array.len(),
                            });
                        }
                        Ok(Command::FlushAll)
                    }

                    _ => Err(CommandError::UnknownCommand(command_name)),
                }
            }
            _ => Err(CommandError::ParseError("expected array".to_string())),
        }
    }
}

fn extract_string(value: &RespValue) -> Result<String, CommandError> {
    match value {
        RespValue::BulkString(Some(s)) => Ok(s.clone()),
        RespValue::SimpleString(s) => Ok(s.clone()),
        _ => Err(CommandError::ParseError("expected string".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get() {
        let input = RespValue::Array(Some(vec![
            RespValue::BulkString(Some("GET".to_string())),
            RespValue::BulkString(Some("mykey".to_string())),
        ]));

        assert_eq!(
            Command::try_from(input).unwrap(),
            Command::Get {
                key: "mykey".to_string()
            }
        );
    }

    #[test]
    fn test_parse_set() {
        let input = RespValue::Array(Some(vec![
            RespValue::BulkString(Some("SET".to_string())),
            RespValue::BulkString(Some("mykey".to_string())),
            RespValue::BulkString(Some("myvalue".to_string())),
        ]));

        assert_eq!(
            Command::try_from(input).unwrap(),
            Command::Set {
                key: "mykey".to_string(),
                value: "myvalue".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_del() {
        let input = RespValue::Array(Some(vec![
            RespValue::BulkString(Some("DEL".to_string())),
            RespValue::BulkString(Some("key1".to_string())),
            RespValue::BulkString(Some("key2".to_string())),
        ]));

        assert_eq!(
            Command::try_from(input).unwrap(),
            Command::Del {
                keys: vec!["key1".to_string(), "key2".to_string()],
            }
        );
    }

    #[test]
    fn test_parse_ping() {
        let input = RespValue::Array(Some(vec![RespValue::BulkString(Some("PING".to_string()))]));

        assert_eq!(Command::try_from(input).unwrap(), Command::Ping);
    }

    #[test]
    fn test_unknown_command() {
        let input = RespValue::Array(Some(vec![RespValue::BulkString(Some(
            "UNKNOWN".to_string(),
        ))]));

        assert!(matches!(
            Command::try_from(input),
            Err(CommandError::UnknownCommand(_))
        ));
    }

    #[test]
    fn test_wrong_number_of_arguments() {
        let input = RespValue::Array(Some(vec![RespValue::BulkString(Some("GET".to_string()))]));

        assert!(matches!(
            Command::try_from(input),
            Err(CommandError::WrongNumberOfArguments { .. })
        ));
    }
}
