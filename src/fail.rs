use std::{
    iter::FromIterator,
    cmp::Ordering,
};
use crate::{
    Index,
    Error,
};

pub enum Fail<E> {
    None,
    One(Index, E),
    Group(Index, Vec<E>),
}

impl<E> Fail<E> {
    pub fn none() -> Self {
        Fail::None
    }

    pub fn one(idx: Index, err: E) -> Self {
        Fail::One(idx, err)
    }

    pub fn map<D>(self, f: impl Fn(E) -> D) -> Fail<D> {
        match self {
            Fail::None => Fail::None,
            Fail::One(idx, err) => Fail::One(idx, f(err)),
            Fail::Group(idx, errs) => Fail::Group(idx, errs.into_iter().map(f).collect()),
        }
    }

    fn furthest_idx(&self) -> Index {
        match self {
            Fail::None => 0,
            Fail::One(idx, _) => *idx,
            Fail::Group(idx, _) => *idx,
        }
    }

    pub fn max<S>(self, other: Self) -> Self
        where E: Error<S>,
    {
        match (self, other) {
            (Fail::One(a_idx, a_err), Fail::One(b_idx, b_err)) => match a_idx.cmp(&b_idx) {
                Ordering::Greater => Fail::One(a_idx, a_err),
                Ordering::Less => Fail::One(b_idx, b_err),
                Ordering::Equal => Fail::One(a_idx, a_err.merge(b_err)),
            }
            (this, other) => if this.furthest_idx() > other.furthest_idx() {
                this
            } else {
                other
            },
        }
    }

    pub fn collect<I: FromIterator<E>>(self) -> I {
        match self {
            Fail::None => I::from_iter(None),
            Fail::One(_, err) => I::from_iter(Some(err)),
            Fail::Group(_, errs) => I::from_iter(errs),
        }
    }
}
