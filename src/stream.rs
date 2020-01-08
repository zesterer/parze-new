use smallbox::{SmallBox, space::S4};
use crate::Index;

trait StreamInner<'a> {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn cloned(&self) -> SmallBox<dyn StreamInner<'a, Item=Self::Item> + 'a, S4>;

}

#[derive(Clone)]
struct Inner<I>(I);

impl<'a, T, I: Iterator<Item=T> + Clone + 'a> StreamInner<'a> for Inner<I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn cloned(&self) -> SmallBox<dyn StreamInner<'a, Item=Self::Item> + 'a, S4> {
        SmallBox::new(self.clone())
    }
}

pub struct Stream<'a, T> {
    idx: Index,
    iter: SmallBox<dyn StreamInner<'a, Item=T> + 'a, S4>,
}

impl<'a, T> Stream<'a, T> {
    pub fn from_iter<I: IntoIterator<Item=T> + 'a>(iter: I) -> Self
        where I::IntoIter: Clone
    {
        Self {
            idx: 0,
            iter: SmallBox::new(Inner(iter.into_iter())),
        }
    }

    pub fn index(&self) -> Index {
        self.idx
    }
}

impl<'a, T> Clone for Stream<'a, T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            iter: self.iter.cloned(),
        }
    }
}

impl<'a, T> Iterator for Stream<'a, T> {
    type Item = (Index, T);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter
            .next()
            .map(|item| (self.idx, item));
        self.idx += 1;
        item
    }
}
