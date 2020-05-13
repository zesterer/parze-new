use std::marker::PhantomData;
use crate::{
    util::attempt,
    Parser,
    Pattern,
    Error,
    Stream,
    ParseResult,
    Fail,
};

// Any

pub fn any<I, E>() -> Parser<impl Pattern<E, Input=I, Output=I>, E>
    where
        I: Clone,
        E: Error<I>,
{
    struct Any<I, E>(PhantomData<(I, E)>);

    impl<I, E> Pattern<E> for Any<I, E>
        where
            I: Into<E::Thing> + Clone,
            E: Error<I>,
    {
        type Input = I;
        type Output = I;

        fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
            attempt(stream, |stream| {
                match stream.next() {
                    Some((_, sym)) => Ok((sym.clone(), Fail::none())),
                    None => Err(Fail::one(!0, E::unexpected_end())),
                }
            })
        }

        fn cloned(&self) -> Self where Self: Sized {
            Self(PhantomData)
        }
    }

    Parser::from_pat(Any(PhantomData))
}

// End

pub fn end<I, E>() -> Parser<impl Pattern<E, Input=I, Output=()>, E>
    where
        I: Clone,
        E: Error<I>
{
    struct End<I, E>(PhantomData<(I, E)>);

    impl<I, E> Pattern<E> for End<I, E>
        where
            I: Into<E::Thing> + Clone,
            E: Error<I>,
    {
        type Input = I;
        type Output = ();

        fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
            let checkpoint = stream.checkpoint();
            attempt(stream, |stream| {
                match stream.next() {
                    Some((idx, sym)) => Err(Fail::one(idx, E::expected_end(sym, stream.span_from(checkpoint)))),
                    None => Ok(((), Fail::none())),
                }
            })
        }

        fn cloned(&self) -> Self where Self: Sized {
            Self(PhantomData)
        }
    }

    Parser::from_pat(End(PhantomData))
}

// Just

pub fn just<I, J, E>(item: J) -> Parser<impl Pattern<E, Input=I, Output=I>, E>
    where
        I: PartialEq<J> + Clone,
        J: Into<E::Thing> + Clone,
        E: Error<I>,
{
    struct Just<I, J, E>(J, PhantomData<(I, E)>);

    impl<I, J, E> Pattern<E> for Just<I, J, E>
        where
            I: PartialEq<J> + Clone,
            J: Into<E::Thing> + Clone,
            E: Error<I>,
    {
        type Input = I;
        type Output = I;

        fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
            let checkpoint = stream.checkpoint();
            attempt(stream, |stream| {
                match stream.next() {
                    Some((_, sym)) if sym == &self.0 => Ok((sym.clone(), Fail::none())),
                    Some((idx, sym)) => Err(Fail::one(idx, E::unexpected_sym(sym, stream.span_from(checkpoint)).expected(self.0.clone().into()))),
                    None => Err(Fail::one(!0, E::unexpected_end())),
                }
            })
        }

        fn cloned(&self) -> Self where Self: Sized {
            Self(self.0.clone(), PhantomData)
        }
    }

    Parser::from_pat(Just(item, PhantomData))
}

// Seq

pub fn seq<I, J, E>(item: impl IntoIterator<Item=J>) -> Parser<impl Pattern<E, Input=I, Output=Vec<I>>, E>
    where
        I: PartialEq<J> + Clone,
        J: Into<E::Thing> + Clone,
        E: Error<I>,
{
    struct Seq<I, J, E>(Vec<J>, PhantomData<(I, E)>);

    impl<I, J, E> Pattern<E> for Seq<I, J, E>
        where
            I: PartialEq<J> + Clone,
            J: Into<E::Thing> + Clone,
            E: Error<I>,
    {
        type Input = I;
        type Output = Vec<I>;

        fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
            let checkpoint = stream.checkpoint();
            attempt(stream, |stream| {
                let mut syms = Vec::new();
                for item in self.0.iter() {
                    match stream.next() {
                        Some((_, sym)) if sym == item => syms.push(sym.clone()),
                        Some((idx, sym)) => return Err(Fail::one(idx, E::unexpected_sym(sym, stream.span_from(checkpoint)).expected(item.clone().into()))),
                        None => return Err(Fail::one(!0, E::unexpected_end())),
                    }
                }
                Ok((syms, Fail::none()))
            })
        }

        fn cloned(&self) -> Self where Self: Sized {
            Self(self.0.clone(), PhantomData)
        }
    }

    Parser::from_pat(Seq(item.into_iter().collect(), PhantomData))
}

// NestedParse

pub fn nested_parse<P, I, Ins, J, E>(f: impl Fn(I) -> Option<(Parser<P, E>, Ins)> + Clone) -> Parser<impl Pattern<E, Input=I, Output=J>, E>
    where
        P: Pattern<E, Input=I, Output=J>,
        I: Clone,
        Ins: IntoIterator<Item=I>,
        Ins::IntoIter: Clone,
        E: Error<I>,
{
    struct NestedParse<F, P, I, Ins, J, E>(F, PhantomData<(P, I, Ins, J, E)>);

    impl<I, Ins, J, F, P, E> Pattern<E> for NestedParse<F, P, I, Ins, J, E>
        where
            I: Clone,
            Ins: IntoIterator<Item=I>,
            Ins::IntoIter: Clone,
            F: Fn(I) -> Option<(Parser<P, E>, Ins)> + Clone,
            P: Pattern<E, Input=I, Output=J>,
            E: Error<I>,
    {
        type Input = I;
        type Output = J;

        fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
            let checkpoint = stream.checkpoint();
            attempt(stream, |stream| {
                match stream.next() {
                    Some((idx, sym)) => match self.0(sym.clone()) {
                        Some((parser, ins)) => match parser.parse_inner(&ins.into_iter().collect::<Vec<_>>()) {
                            Ok((out, _)) => Ok((out, Fail::none())),
                            Err(err) => Err(err.add_index(idx)), // This is a total hack
                        },
                        None => Err(Fail::one(idx, E::unexpected_sym(sym, stream.span_from(checkpoint)))),
                    },
                    None => Err(Fail::one(!0, E::unexpected_end())),
                }
            })
        }

        fn cloned(&self) -> Self where Self: Sized {
            Self(self.0.clone(), PhantomData)
        }
    }

    Parser::from_pat(NestedParse(f, PhantomData))
}

// PermitMap

pub fn permit_map<I, J, E>(f: impl Fn(I) -> Option<J> + Clone) -> Parser<impl Pattern<E, Input=I, Output=J>, E>
    where
        I: Clone,
        E: Error<I>,
{
    struct PermitMap<F, I, J, E>(F, PhantomData<(I, J, E)>);

    impl<I, J, F, E> Pattern<E> for PermitMap<F, I, J, E>
        where
            I: Clone,
            F: Fn(I) -> Option<J> + Clone,
            E: Error<I>,
    {
        type Input = I;
        type Output = J;

        fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
            let checkpoint = stream.checkpoint();
            attempt(stream, |stream| {
                match stream.next() {
                    Some((idx, sym)) => match self.0(sym.clone()) {
                        Some(out) => Ok((out, Fail::none())),
                        None => Err(Fail::one(idx, E::unexpected_sym(sym, stream.span_from(checkpoint)))),
                    },
                    None => Err(Fail::one(!0, E::unexpected_end())),
                }
            })
        }

        fn cloned(&self) -> Self where Self: Sized {
            Self(self.0.clone(), PhantomData)
        }
    }

    Parser::from_pat(PermitMap(f, PhantomData))
}

// Permit

pub fn permit<I, E>(f: impl Fn(&I) -> bool + Clone) -> Parser<impl Pattern<E, Input=I, Output=I>, E>
    where
        I: Clone,
        E: Error<I>,
{
    permit_map(move |sym| if f(&sym) { Some(sym) } else { None })
}
