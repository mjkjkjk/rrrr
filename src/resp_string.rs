use log::debug;

use crate::{Command, CommandType};

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
            "INCR" => Command {
                kind: CommandType::Incr,
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
