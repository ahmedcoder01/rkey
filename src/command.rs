use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::RespType;
use crate::Storage;


pub struct CommandHandler {
    storage: Arc<Mutex<Storage>>
}


impl CommandHandler {

    pub fn new(storage: Arc<Mutex<Storage>>) -> Self {
        Self { storage }
    }

    pub fn handle_cmd(&mut self, d: RespType) -> Result<RespType, CommandErr> {
        let RespType::Array(parts) = d else { 
            return Err(CommandErr::InvalidArgs("No command provided".to_string()));
        };

         let RespType::BString(cmd_name) = &parts[0] else {
            return Err(CommandErr::InvalidArgs("Invalid command".to_string()));
         };

         if cmd_name == "PING" {
            return Ok(RespType::String("PONG".to_string()))
         }

         let cmd = Self::match_cmd(cmd_name).ok_or_else(|| CommandErr::UnknownCommand(cmd_name.to_string()))?;
         

         let (valid_args_num, expected) = cmd.validate_args(&parts[1..]);

        if valid_args_num {
            cmd.execute(&parts[1..], Arc::clone(&self.storage)) // Execute the command
        } else {
            Err(CommandErr::InvalidArgs(format!("Expected {}. Got {}", expected, &parts[1..].len())))
        }

         
    }



    fn match_cmd(cmd_name: &str) -> Option<Box<dyn Command>> {
        match cmd_name {
            "SET" => Some(Box::new(Set::new())),
            "GET" => Some(Box::new(Get::new())),
            "DEL" => Some(Box::new(Del::new())),
            "COMMAND" => Some(Box::new(CommandCmd::new())),
            _ => None
        }
    }
}

trait Command<'a> {
    
    fn execute(&self, parts: &'a [RespType], storage: Arc<Mutex<Storage>>) -> Result<RespType, CommandErr>;

    fn validate_args(&self, parts: &'a [RespType]) -> (bool, u8);
}


struct CommandCmd;

impl CommandCmd {
    fn new() -> Self {Self{}}
}

impl<'a> Command<'a> for CommandCmd {
    fn execute(&self, parts: &'a [RespType], storage: Arc<Mutex<Storage>>) -> Result<RespType, CommandErr> {
        Ok(RespType::String("OK".to_string()))
    }

    fn validate_args(&self, parts: &'a [RespType]) -> (bool, u8) {
        (true, 0)
    }
}

struct Set;

impl Set {
    fn new() -> Self {Self {}}
}

impl<'a> Command<'a> for Set {
    

    fn execute(&self, parts: &'a [RespType], storage: Arc<Mutex<Storage>>) -> Result<RespType, CommandErr> {
        let RespType::BString(k) = &parts[0] else {
            return Err(CommandErr::InvalidArgs("wrong key format".to_string()));
        };

        let RespType::BString(v) = &parts[1] else {
            return Err(CommandErr::InvalidArgs("wrong value format".to_string()));
        };

        storage.lock().unwrap().set(k.as_str(), v.as_str());
        Ok(RespType::String("OK".to_string()))
    }

    fn validate_args(&self ,parts: &'a [RespType]) -> (bool, u8) {
        (parts.len() == 2, 2)
    }
}

struct Get;

impl Get {
    fn new() -> Self {Self {}}
}

impl<'a> Command<'a> for Get {
    

    fn execute(&self, parts: &'a [RespType], storage: Arc<Mutex<Storage>>) -> Result<RespType, CommandErr> {
        let RespType::BString(k) = &parts[0] else {
            return Err(CommandErr::InvalidArgs("wrong key format".to_string()));
        };

        let value = storage.lock().unwrap().get(k).cloned();

        match value {
            Some(v) => {
                Ok(RespType::BString(v))
            },
            None => {
                Ok(RespType::Null)
            }
        }
    }

    fn validate_args(&self ,parts: &'a [RespType]) -> (bool, u8) {
        (parts.len() == 1, 1)
    }
}


struct Del;

impl Del {
    fn new() -> Self {Self {}}
}

impl<'a> Command<'a> for Del {
    

    fn execute(&self, parts: &'a [RespType], storage: Arc<Mutex<Storage>>) -> Result<RespType, CommandErr> {
        
        let mut c = 0;
        for key in parts {
            if let RespType::BString(k) = key  {
                let deleted  = storage.lock().unwrap().del(k);
                if deleted {c += 1}
            }
        }

        Ok(RespType::Int(c))
    }

    fn validate_args(&self ,parts: &'a [RespType]) -> (bool, u8) {
        (parts.len() >= 1, 1)
    }
}



#[derive(Debug)]
pub enum CommandErr {
    InvalidArgs(String),
    UnknownCommand(String),
}

impl std::fmt::Display for CommandErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandErr::InvalidArgs(msg) => write!(f, "Invalid arguments: {}", msg),
            CommandErr::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
        }
    }
}

