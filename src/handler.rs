pub mod Handler {
    use crate::resp::{Value, ValueString};

    type Command = fn(Vec<Value>) -> Value;

    fn ping(values: Vec<Value>) -> Value {
        Value::ValueString(ValueString {
            str: "PONG".to_string(),
        })
    }

    pub fn from_command(command: String) -> Command {
        match command.as_str() {
            "PING" => ping,
            _ => panic!("unexpected command"),
        }
    }
}
