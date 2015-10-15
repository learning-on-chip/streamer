use std::f64::INFINITY;

#[link_name = "m"]
extern {
    fn nextafter(x: f64, y: f64) -> f64;
}

#[inline]
pub fn next_after(x: f64) -> f64 {
    unsafe { nextafter(x, INFINITY) }
}
