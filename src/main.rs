fn main() {}

#[cfg(test)]
mod tests {
    use crate::RespCommand;

    #[test]
    fn convert_simple_ping() {
        let raw_command = "PING".to_string();
        let expected = "*1\r\n$4\r\nPING\r\n";
        assert_eq!(
            RespCommand::from_string(raw_command).to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn convert_ping_with_one_argument() {
        let raw_command = "PING \"test\"".to_string();
        let expected = "*2\r\n$4\r\nPING\r\n$6\r\n\"test\"\r\n";
        assert_eq!(
            RespCommand::from_string(raw_command).to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn convert_ping_with_multiple_arguments() {
        let raw_command = "PING \"test\" \"another\"".to_string();
        let expected = "*3\r\n$4\r\nPING\r\n$6\r\n\"test\"\r\n$9\r\n\"another\"\r\n";
        assert_eq!(
            RespCommand::from_string(raw_command).to_string(),
            expected.to_string()
        );
    }
}

pub struct RespCommand {
    raw_str: String,
}

impl RespCommand {
    pub fn from_string(s: String) -> Self {
        let parts = s.split(" ");
        let collected = parts.collect::<Vec<&str>>();
        let prefix = format!("*{}\r\n", collected.len());
        let content = collected
            .iter()
            .map(|part| format!("${}\r\n{}\r\n", part.len(), part))
            .fold("".to_string(), |acc, x| acc + &x);

        RespCommand {
            raw_str: prefix + &content,
        }
    }
}

impl ToString for RespCommand {
    fn to_string(&self) -> String {
        self.raw_str.to_owned()
    }
}
