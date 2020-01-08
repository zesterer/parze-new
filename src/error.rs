use std::{
    marker::PhantomData,
    fmt,
    hash::Hash,
    collections::HashSet,
};
use crate::Index;

pub trait Error<S>: Sized {
    type Context;

    fn unexpected_sym(sym: S, at: Index) -> Self;
    fn unexpected_end() -> Self;
    fn expected_end(sym: S, at: Index) -> Self;
    fn expected(self, _sym: S) -> Self { self }
    fn merge(self, _other: Self) -> Self { self }
    fn context(self, _ctx: Self::Context) -> Self { self }
}

pub type DefaultError<S> = EmptyError<S>;

// EmptyError

#[derive(PartialEq)]
pub struct EmptyError<S>(PhantomData<S>);

impl<S> Error<S> for EmptyError<S> {
    type Context = ();

    fn unexpected_sym(_sym: S, _at: Index) -> Self {
        Self(PhantomData)
    }

    fn unexpected_end() -> Self {
        Self(PhantomData)
    }

    fn expected_end(_sym: S, _at: Index) -> Self {
        Self(PhantomData)
    }

    fn expected(self, _sym: S) -> Self {
        Self(PhantomData)
    }

    fn merge(self, _other: Self) -> Self {
        self
    }
}

impl<S> Clone for EmptyError<S> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<S> fmt::Debug for EmptyError<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmptyError")
    }
}

// SimpleError

#[derive(Clone)]
pub struct SimpleError<S> {
    found: Option<S>,
    at: Option<Index>,
    expected: Option<HashSet<S>>,
}

impl<S: Hash + Eq> Error<S> for SimpleError<S> {
    type Context = ();

    fn unexpected_sym(sym: S, at: Index) -> Self {
        Self {
            found: Some(sym),
            at: Some(at),
            expected: Some(HashSet::default()),
        }
    }

    fn unexpected_end() -> Self {
        Self {
            found: None,
            at: None,
            expected: Some(HashSet::default()),
        }
    }

    fn expected_end(sym: S, at: Index) -> Self {
        Self {
            found: Some(sym),
            at: Some(at),
            expected: None,
        }
    }

    fn expected(mut self, sym: S) -> Self {
        self.expected.as_mut().map(|e| e.insert(sym));
        self
    }

    fn merge(self, other: Self) -> Self {
        Self {
            found: self.found.or(other.found),
            at: self.at,
            expected: if self.expected.is_none() && other.expected.is_none() {
                None
            } else {
                Some(self.expected.map(|e| e.into_iter()).into_iter().flatten()
                    .chain(other.expected.map(|e| e.into_iter()).into_iter().flatten())
                    .collect())
            },
        }
    }
}

impl<S: Hash + Eq + fmt::Debug> fmt::Debug for SimpleError<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.found {
            Some(found) => write!(f, "Found {:?}", found)?,
            None => write!(f, "Found end of input")?,
        }
        match &self.expected {
            Some(expected) => write!(f, ", expected {:?}", expected),
            None => write!(f, ", expected end of input"),
        }
    }
}
