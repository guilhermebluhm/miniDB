use std::fmt;

#[derive(Debug)]
pub enum TypeErrorCommand{
    Parser(String)
}

impl fmt::Display for TypeErrorCommand{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self {
            TypeErrorCommand::Parser(msg) => write!(f, "{}", msg),
        }
    }
}