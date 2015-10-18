use std::str::FromStr;

use {Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Element {
    pub id: usize,
    pub kind: ElementKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementKind {
    Core,
    L3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementCapacity {
    Single,
    Infinite,
}

impl Element {
    #[inline(always)]
    pub fn capacity(&self) -> ElementCapacity {
        if self.kind == ElementKind::Core {
            ElementCapacity::Single
        } else {
            ElementCapacity::Infinite
        }
    }
}

impl FromStr for ElementKind {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        let lower = id.to_lowercase();
        if lower.starts_with("core") {
            return Ok(ElementKind::Core);
        } else if lower.starts_with("l3") {
            return Ok(ElementKind::L3);
        }
        raise!("found an unknown element id ({:?})", id);
    }
}
