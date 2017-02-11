
pub trait Zero {
    fn zero() -> Self;
}

impl Zero for usize {
    fn zero() -> usize {
        0
    }
}
