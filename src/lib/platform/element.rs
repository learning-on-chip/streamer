use std::str::FromStr;

use {Error, Result};

/// A processing element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Element {
    /// The identifier.
    pub id: usize,
    /// The type.
    pub kind: ElementKind,
}

/// The type of a processing element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementKind {
    /// A central processing unit.
    Core,
    /// An L3 cache.
    L3,
}

/// The capacity of a processing element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElementCapacity {
    /// Capable of hosting only one job at a time.
    Single,
    /// Capable of hosting as many jobs at a time as needed.
    Infinite,
}

impl Element {
    /// Return the capacity of the processing element.
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
