use std::sync::{Arc, Mutex};

use log::debug;

use crate::{command::Command, resp::RespValue, storage::Storage};

pub fn handle_command(command: Command, storage: &Arc<Mutex<Storage>>) -> RespValue {
    match command {
        Command::Ping => RespValue::SimpleString("PONG".to_string()),

        Command::Get { key } => {
            let mut storage = storage.lock().unwrap();
            match storage.get(key) {
                Some(value) => RespValue::BulkString(Some(value.to_string().clone())),
                None => RespValue::BulkString(None),
            }
        }

        Command::Set { key, value } => {
            let mut storage = storage.lock().unwrap();
            storage.set(key, value);
            RespValue::SimpleString("OK".to_string())
        }

        Command::Del { keys } => {
            println!("Got DEL command for keys: {:?}", keys);
            let mut storage = storage.lock().unwrap();
            for key in keys {
                storage.del(key);
            }
            RespValue::SimpleString("OK".to_string())
        }

        Command::CommandDocs => {
            println!("Got COMMAND DOCS command");
            RespValue::SimpleString("OK".to_string()) // Placeholder response
        }

        Command::IncrBy { key, value } => {
            let mut storage = storage.lock().unwrap();
            match handle_numeric_operation(&mut storage, key, value.parse::<i64>(), |n, incr| {
                n + incr
            }) {
                Ok(new_value) => RespValue::Integer(new_value),
                Err(err_msg) => RespValue::Error(err_msg),
            }
        }

        Command::Incr { key } => {
            let mut storage = storage.lock().unwrap();
            match handle_numeric_operation(&mut storage, key, Ok(1), |n, _| n + 1) {
                Ok(new_value) => RespValue::Integer(new_value),
                Err(err_msg) => RespValue::Error(err_msg),
            }
        }

        Command::DecrBy { key, value } => {
            let mut storage = storage.lock().unwrap();
            match handle_numeric_operation(&mut storage, key, value.parse::<i64>(), |n, decr| {
                n - decr
            }) {
                Ok(new_value) => RespValue::Integer(new_value),
                Err(err_msg) => RespValue::Error(err_msg),
            }
        }

        Command::Decr { key } => {
            let mut storage = storage.lock().unwrap();
            match handle_numeric_operation(&mut storage, key, Ok(1), |n, _| n - 1) {
                Ok(new_value) => RespValue::Integer(new_value),
                Err(err_msg) => RespValue::Error(err_msg),
            }
        }
        Command::MGet { keys } => {
            let mut storage = storage.lock().unwrap();
            let values: Vec<RespValue> = keys
                .iter()
                .map(|key| match storage.get(key.to_string()) {
                    Some(value) => RespValue::BulkString(Some(value.clone())),
                    None => RespValue::BulkString(None),
                })
                .collect();
            if values.len() == 1 {
                values.into_iter().next().unwrap()
            } else {
                RespValue::Array(Some(values))
            }
        }
        Command::FlushAll => {
            let mut storage = storage.lock().unwrap();
            storage.clear();
            RespValue::SimpleString("OK".to_string())
        }
        Command::Exists { keys } => {
            let storage = storage.lock().unwrap();
            let count = keys
                .iter()
                .filter(|key| storage.has(key.to_string()))
                .count();
            RespValue::Integer(count as i64)
        }
        Command::Expire { key, expire } => {
            let mut storage = storage.lock().unwrap();
            let Ok(ttl) = expire.parse::<i64>() else {
                return RespValue::Error("value is not an integer or out of range".to_string());
            };
            if !storage.has(key.clone()) {
                return RespValue::SimpleString("0".to_string());
            }
            storage.set_expire(key, ttl);
            RespValue::SimpleString("1".to_string())
        }
        Command::Persist { key } => {
            let mut storage = storage.lock().unwrap();
            let result = storage.remove_expire(key);
            match result {
                Ok(_) => RespValue::SimpleString("1".to_string()),
                Err(_) => RespValue::SimpleString("0".to_string()),
            }
        }
        Command::Keys { pattern } => {
            debug!("Got KEYS command for pattern: {}", pattern);
            let storage = storage.lock().unwrap();
            let keys = storage.keys(pattern);
            debug!("Found keys: {:?}", keys);
            RespValue::Array(Some(
                keys.iter()
                    .map(|k| RespValue::BulkString(Some(k.clone())))
                    .collect(),
            ))
        }
        Command::TTL { key } => {
            let storage = storage.lock().unwrap();
            let ttl = storage.get_ttl(key);
            RespValue::Integer(ttl)
        }
    }
}

fn handle_numeric_operation(
    storage: &mut std::sync::MutexGuard<Storage>,
    key: String,
    value: Result<i64, std::num::ParseIntError>,
    operation: impl FnOnce(i64, i64) -> i64,
) -> Result<i64, String> {
    let value = value.map_err(|_| "ERR value is not an integer or out of range".to_string())?;

    let default = "0".to_string();
    let current_value = storage.get(key.clone()).unwrap_or(default);

    let current_num = current_value
        .parse::<i64>()
        .map_err(|_| "ERR value is not an integer or out of range".to_string())?;
    let new_value = operation(current_num, value);
    storage.set(key, new_value.to_string());

    Ok(new_value)
}
