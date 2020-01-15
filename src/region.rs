use std::ops::Range;

pub trait Region<T> {
    fn none() -> Self;
    fn single(index: usize, sym: &T) -> Self;
    fn group(syms: &[T], range: Range<usize>) -> Self;
}

impl<T> Region<T> for Option<Range<usize>> {
    fn none() -> Self {
        None
    }

    fn single(index: usize, _sym: &T) -> Self {
        Some(index..index + 1)
    }

    fn group(_syms: &[T], range: Range<usize>) -> Self {
        Some(range)
    }
}
