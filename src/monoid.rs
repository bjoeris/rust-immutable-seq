
pub trait Monoid : Sized {
    fn id() -> Self;
    fn op(self, other: Self) -> Self;
    fn fold<I>(values: I) -> Self
        where I: IntoIterator<Item=Self> {
        let mut iter = values.into_iter();
        match iter.next() {
            None => Self::id(),
            Some(x0) => {
                iter.fold(x0, |res, x| res.op(x))
            }
        }
    }
}

impl Monoid for usize {
    fn id() -> Self { 0 }
    fn op(self, other: Self) -> Self { self + other }
}
