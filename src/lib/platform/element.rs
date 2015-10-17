use std::str::FromStr;

use {Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Element {
    pub id: usize,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Core,
    L3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capacity {
    Single,
    Infinite,
}

impl Element {
    #[inline(always)]
    pub fn capacity(&self) -> Capacity {
        if self.kind == Kind::Core {
            Capacity::Single
        } else {
            Capacity::Infinite
        }
    }
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        let lower = id.to_lowercase();
        if lower.starts_with("core") {
            return Ok(Kind::Core);
        } else if lower.starts_with("l3") {
            return Ok(Kind::L3);
        }
        raise!("found an unknown id {:?}", id);
    }
}
