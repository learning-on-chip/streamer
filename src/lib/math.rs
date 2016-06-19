#[link_name = "m"]
extern {
    fn nextafter(x: f64, y: f64) -> f64;
}

#[inline]
pub fn next_after(x: f64) -> f64 {
    use std::f64::INFINITY;
    unsafe { nextafter(x, INFINITY) }
}
