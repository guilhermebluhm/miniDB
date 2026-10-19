use crate::enums::TypeErrorCommand::TypeErrorCommand;
use regex::Regex;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Command{
    SET {key: String, value: String},
    GET {key: String},
    DELETE {key: String},
    EXIT
}

pub fn ParseCommand(value: &mut String) -> Result<Command, TypeErrorCommand> {
    
    let re = Regex::new(r"[^\p{L}\p{N}]+").unwrap();
    let raw_data = re.replace_all(value.as_str(), " ").to_string();
    let commands = raw_data.split(" ").map(|f| f.to_string()).collect::<Vec<String>>();

    let op = commands.get(0).unwrap().as_str();

    if op.contains("SET") && commands.len() <= 2{
        return Err(TypeErrorCommand::Parser("Failed to mount SET operation insuficient arguments".to_string()));
    }

    let mut command = Command::EXIT;
    
    if !commands.is_empty() {
        match commands.get(0).unwrap().as_str() { 
            "GET" => {
                command = Command::GET {key: commands.get(1).unwrap().to_string()};
            }
            "SET" => {
                command = Command::SET {key: commands.get(1).unwrap().to_string(), value: commands.get(2).unwrap().to_string()};
            }
            "DELETE" => {
                command = Command::DELETE {key: commands.get(1).unwrap().to_string()};
            }
            _ => {}
        }   
    }

    if command == Command::EXIT {
        return Err(TypeErrorCommand::Parser("Failed to parse command".to_string()));
    }
    
    Ok(command)
    
}