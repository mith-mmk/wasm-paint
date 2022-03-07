use self::JPEGWorning::*;
use self::WorningKind::*;


pub enum JPEGWorning {
      Simple(WorningKind),
      SimpleAddMessage(WorningKind,String),
      Custom(String),
  }

impl JPEGWorning {
    pub fn fmt(&self) -> String {
        match self {
            Simple(error_kind) => { error_kind.as_str().to_string()},
            SimpleAddMessage(error_kind,s) => {
                error_kind.as_str().to_string() + " " + &s.to_string()
            },
            Custom(s) => {s.to_string()},
        }
    }
}

pub enum WorningKind {
      IlligalRSTMaker,
      UnfindEOIMaker,
      DataCorruption,
      BufferOverrun,
  }

  impl WorningKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match &*self {
            IlligalRSTMaker => {"Illigal RST Maker"},
            OutOfMemory => {"Out of memory"},
            DataCorruption => {"Data Corruption"},
            BufferOverrun => {"Buffer Overrun"},
        }
    }
}