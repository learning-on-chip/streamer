use std::collections::HashMap;
use std::sync::{Once, ONCE_INIT};
use std::{fmt, mem};

type Map = HashMap<&'static str, usize>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ID(usize);

impl ID {
    pub fn new(name: &'static str) -> ID {
        static mut DATA: *mut Map = 0 as *mut _;
        static mut ONCE: Once = ONCE_INIT;
        unsafe {
            ONCE.call_once(|| {
                DATA = mem::transmute::<Box<Map>, _>(Box::new(HashMap::new()));
            });
            let map = &mut *DATA;
            let count = map.entry(name).or_insert(0);
            let id = *count;
            *count += 1;
            ID(id)
        }
    }

    #[inline(always)]
    pub fn number(&self) -> usize {
        self.0
    }
}

impl fmt::Display for ID {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "#{}", self.0)
    }
}
