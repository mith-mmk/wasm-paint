use std::fmt::Display;
use std::fmt::Debug;

pub struct Error {
   pub message: String, 
}

impl std::error::Error for Error {

}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        write!(f,"{}",&self.message)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> { 
        write!(f,"{}",&self.message)
    }
}

