#![feature(trait_alias, try_trait)]

pub mod error;
pub mod stream;
pub mod span;
pub mod primitives;
pub mod ops;
pub mod declare;
mod fail;
mod util;

use std::marker::PhantomData;
use crate::{
    error::*,
    stream::*,
    fail::*,
};

pub type Index = u64;

pub type ParseResult<O, E> = Result<(O, Fail<E>), Fail<E>>;

pub trait Pattern<E> {
    type Input;
    type Output;

    // Should leave `stream` in its original state upon failure
    fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E>;

    fn cloned(&self) -> Self where Self: Sized;
}

pub struct Parser<P, E> {
    pat: P,
    phantom: PhantomData<E>,
}

impl<E, P: Pattern<E>> Clone for Parser<P, E> {
    fn clone(&self) -> Self {
        Self {
            pat: self.pat.cloned(),
            phantom: PhantomData,
        }
    }
}

impl<P, E> Parser<P, E> {
    fn from_pat(pat: P) -> Self {
        Self {
            pat,
            phantom: PhantomData,
        }
    }

    fn parse_inner(&self, inputs: &[P::Input]) -> ParseResult<P::Output, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        self.pat.parse(&mut Stream::from(inputs))
    }

    pub fn parse<I>(&self, inputs: I) -> Result<P::Output, Vec<E>>
        where
            P: Pattern<E>,
            I: IntoIterator<Item=P::Input>,
            I::IntoIter: Clone,
            E: Error<P::Input>,
    {
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        match self.parse_inner(&inputs) {
            Ok((out, _)) => Ok(out),
            Err(fail) => Err(fail.collect()),
        }
    }

    pub fn boxed(self) -> Parser<impl Pattern<E, Input=P::Input, Output=P::Output>, E>
        where
            P: Pattern<E> + 'static,
            E: Error<P::Input>,
    {
        trait InnerPattern<I, O, E> {
            fn pat(&self) -> &dyn Pattern<E, Input=I, Output=O>;
            fn cloned(&self) -> Box<dyn InnerPattern<I, O, E>>;
        }

        struct BoxedPattern<P>(P);

        impl<P, I, O, E> InnerPattern<I, O, E> for BoxedPattern<P>
            where P: Pattern<E, Input=I, Output=O> + 'static
        {
            fn pat(&self) -> &dyn Pattern<E, Input=I, Output=O> {
                &self.0
            }

            fn cloned(&self) -> Box<dyn InnerPattern<I, O, E>> {
                Box::new(Self(self.0.cloned()))
            }
        }

        struct Boxed<I, O, E>(Box<dyn InnerPattern<I, O, E>>);

        impl<I, O, E> Pattern<E> for Boxed<I, O, E> {
            type Input = I;
            type Output = O;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                self.0.pat().parse(stream)
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned())
            }
        }

        Parser::from_pat(Boxed(Box::new(BoxedPattern(self.pat))))
    }
}

pub mod prelude {
    pub use crate::{
        Parser,
        Pattern,
        Index,
        primitives::*,
        declare::*,
        error::DefaultError,
    };
}
