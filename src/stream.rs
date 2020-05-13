use crate::{
    Index,
    span::Span,
};

pub struct Stream<'a, T> {
    slice: &'a [T],
    index: usize,
}

impl<'a, T> Copy for Stream<'a, T> {}

impl<'a, T> Clone for Stream<'a, T> {
    fn clone(&self) -> Self {
        Self {
            slice: self.slice,
            index: self.index,
        }
    }
}

impl<'a, T> From<&'a [T]> for Stream<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, T> Stream<'a, T> {
    pub fn checkpoint(&self) -> Index {
        self.index as Index
    }

    pub fn span_from<R: Span<T>>(&self, checkpoint: Index) -> R {
        let checkpoint = checkpoint as usize;
        Span::group(&self.slice[checkpoint..self.index], checkpoint..self.index)
    }
}

impl<'a, T> Iterator for Stream<'a, T> {
    type Item = (Index, &'a T);

    fn next(&mut self) -> Option<(Index, &'a T)> {
        if let Some(next) = self.slice.get(self.index..).and_then(|s| s.first()) {
            let index = self.index;
            self.index += 1;
            Some((index as Index, next))
        } else {
            None
        }
    }
}
