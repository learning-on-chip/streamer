use std::collections::HashMap;
use std::mem;
use std::sync::{Once, ONCE_INIT};

type Map = HashMap<&'static str, usize>;

pub fn generate(name: &'static str) -> usize {
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
        id
    }
}
