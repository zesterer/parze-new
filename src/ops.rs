use std::marker::PhantomData;
use crate::{
    util::attempt,
    Parser,
    Pattern,
    Error,
    Stream,
    ParseResult,
};

impl<P, E> Parser<P, E> {
    pub fn map<U>(self, f: impl Fn(P::Output) -> U + Clone) -> Parser<impl Pattern<E, Input=P::Input, Output=U>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct Map<A, F, E>(A, F, PhantomData<E>);

        impl<I, E, A, F, X, U> Pattern<E> for Map<A, F, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
                F: Fn(X) -> U + Clone,
        {
            type Input = I;
            type Output = U;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                let (out, fail) = self.0.parse(stream)?;
                Ok(((self.1)(out), fail))
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), self.1.clone(), PhantomData)
            }
        }

        Parser::from_pat(Map(self.pat, f, PhantomData))
    }

    pub fn map_with_span<U>(self, f: impl Fn(P::Output, E::Span) -> U + Clone) -> Parser<impl Pattern<E, Input=P::Input, Output=U>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct MapWithRange<A, F, E>(A, F, PhantomData<E>);

        impl<I, E, A, F, X, U> Pattern<E> for MapWithRange<A, F, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
                F: Fn(X, E::Span) -> U + Clone,
        {
            type Input = I;
            type Output = U;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                let checkpoint = stream.checkpoint();
                let (out, fail) = self.0.parse(stream)?;
                Ok(((self.1)(out, stream.span_from(checkpoint)), fail))
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), self.1.clone(), PhantomData)
            }
        }

        Parser::from_pat(MapWithRange(self.pat, f, PhantomData))
    }

    pub fn chained(self) -> Parser<impl Pattern<E, Input=P::Input, Output=Vec<P::Output>>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        self.map(|out| vec![out])
    }

    pub fn reduce_left<A, B>(self, f: impl Fn(A, B) -> A + Clone) -> Parser<impl Pattern<E, Input=P::Input, Output=A>, E>
        where
            P: Pattern<E, Output=(A, Vec<B>)>,
            E: Error<P::Input>,
    {
        self.map(move |(init, items)| items.into_iter().fold(init, |a, b| f(a, b)))
    }

    pub fn reduce_right<A, B>(self, f: impl Fn(A, B) -> B + Clone) -> Parser<impl Pattern<E, Input=P::Input, Output=B>, E>
        where
            P: Pattern<E, Output=(Vec<A>, B)>,
            E: Error<P::Input>,
    {
        self.map(move |(items, init)| items.into_iter().rev().fold(init, |b, a| f(a, b)))
    }

    pub fn to<U>(self, out: U) -> Parser<impl Pattern<E, Input=P::Input, Output=U>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
            U: Clone,
    {
        self.map(move |_| out.clone())
    }

    pub fn map_err<D>(self, f: impl Fn(E) -> D + Clone) -> Parser<impl Pattern<D, Input=P::Input, Output=P::Output>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
            D: Error<P::Input>,
    {
        struct MapErr<A, F, E>(A, F, PhantomData<E>);

        impl<I, E, A, F, X, D> Pattern<D> for MapErr<A, F, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
                F: Fn(E) -> D + Clone,
        {
            type Input = I;
            type Output = X;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, D> {
                let (out, fail) = self.0.parse(stream).map_err(|fail| fail.map(&self.1))?;
                Ok((out, fail.map(&self.1)))
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), self.1.clone(), PhantomData)
            }
        }

        Parser::from_pat(MapErr(self.pat, f, PhantomData))
    }

    pub fn context(self, ctx: E::Context) -> Parser<impl Pattern<E, Input=P::Input, Output=P::Output>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
            E::Context: Clone,
    {
        self.map_err(move |err| err.context(ctx.clone()))
    }

    pub fn then<Y>(self, other: Parser<impl Pattern<E, Input=P::Input, Output=Y>, E>) -> Parser<impl Pattern<E, Input=P::Input, Output=(P::Output, Y)>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct Then<A, B, E>(A, B, PhantomData<E>);

        impl<I, E, A, B, X, Y> Pattern<E> for Then<A, B, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
                B: Pattern<E, Input=I, Output=Y>,
        {
            type Input = I;
            type Output = (X, Y);

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                attempt(stream, |stream| {
                    let (a, f) = self.0.parse(stream)?;
                    let (b, g) = match self.1.parse(stream) {
                        Ok((b, g)) => (b, g),
                        Err(g) => return Err(f.max(g)),
                    };
                    Ok(((a, b), f.max(g)))
                })
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), self.1.cloned(), PhantomData)
            }
        }

        Parser::from_pat(Then(self.pat, other.pat, PhantomData))
    }

    pub fn chain<O>(self, other: Parser<impl Pattern<E, Input=P::Input, Output=Vec<O>>, E>) -> Parser<impl Pattern<E, Input=P::Input, Output=Vec<O>>, E>
        where
            P: Pattern<E, Output=Vec<O>>,
            E: Error<P::Input>,
    {
        self.then(other).map(|(mut a, mut b)| {
            a.append(&mut b);
            a
        })
    }

    pub fn or(self, other: Parser<impl Pattern<E, Input=P::Input, Output=P::Output>, E>) -> Parser<impl Pattern<E, Input=P::Input, Output=P::Output>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct Or<A, B, E>(A, B, PhantomData<E>);

        impl<I, E, A, B, X> Pattern<E> for Or<A, B, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
                B: Pattern<E, Input=I, Output=X>,
        {
            type Input = I;
            type Output = X;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                match self.0.parse(stream) {
                    Ok((out, fail)) => Ok((out, fail)),
                    Err(a_fail) => match self.1.parse(stream) {
                        Ok((out, b_fail)) => Ok((out, a_fail.max(b_fail))),
                        Err(b_fail) => Err(a_fail.max(b_fail)),
                    },
                }
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), self.1.cloned(), PhantomData)
            }
        }

        Parser::from_pat(Or(self.pat, other.pat, PhantomData))
    }

    pub fn padding_for<Y>(self, other: Parser<impl Pattern<E, Input=P::Input, Output=Y>, E>) -> Parser<impl Pattern<E, Input=P::Input, Output=Y>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        self.then(other).map(|(_, b)| b)
    }

    pub fn padded_by<Y>(self, other: Parser<impl Pattern<E, Input=P::Input, Output=Y>, E>) -> Parser<impl Pattern<E, Input=P::Input, Output=P::Output>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        self.then(other).map(|(a, _)| a)
    }

    pub fn repeated(self) -> Parser<impl Pattern<E, Input=P::Input, Output=Vec<P::Output>>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct Repeated<A, E>(A, PhantomData<E>);

        impl<I, E, A, X> Pattern<E> for Repeated<A, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
        {
            type Input = I;
            type Output = Vec<X>;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                let mut outputs = Vec::new();

                loop {
                    match self.0.parse(stream) {
                        Ok((out, _)) => outputs.push(out),
                        Err(fail) => break Ok((outputs, fail)),
                    }
                }
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), PhantomData)
            }
        }

        Parser::from_pat(Repeated(self.pat, PhantomData))
    }

    pub fn separated_by<Y>(self, other: Parser<impl Pattern<E, Input=P::Input, Output=Y>, E>) -> Parser<impl Pattern<E, Input=P::Input, Output=Vec<P::Output>>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct SeparatedBy<A, B, E>(A, B, PhantomData<E>);

        impl<I, E, A, B, X> Pattern<E> for SeparatedBy<A, B, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
                B: Pattern<E, Input=I>,
        {
            type Input = I;
            type Output = Vec<X>;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                let mut outputs = Vec::new();

                for _ in 0.. {
                    match self.0.parse(stream) {
                        Ok((out, _)) => outputs.push(out),
                        Err(fail) => return Ok((outputs, fail)),
                    }

                    if let Err(fail) = self.1.parse(stream) {
                        return Ok((outputs, fail));
                    }
                }

                unreachable!()
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), self.1.cloned(), PhantomData)
            }
        }

        Parser::from_pat(SeparatedBy(self.pat, other.pat, PhantomData))
    }

    pub fn once_or_more(self) -> Parser<impl Pattern<E, Input=P::Input, Output=Vec<P::Output>>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct OnceOrMore<A, E>(A, PhantomData<E>);

        impl<I, E, A, X> Pattern<E> for OnceOrMore<A, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
        {
            type Input = I;
            type Output = Vec<X>;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                let mut outputs = Vec::new();

                loop {
                    match self.0.parse(stream) {
                        Ok((out, _)) => outputs.push(out),
                        Err(fail) if outputs.len() > 0 => return Ok((outputs, fail)),
                        Err(fail) => return Err(fail),
                    }
                }
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), PhantomData)
            }
        }

        Parser::from_pat(OnceOrMore(self.pat, PhantomData))
    }

    pub fn or_not(self) -> Parser<impl Pattern<E, Input=P::Input, Output=Option<P::Output>>, E>
        where
            P: Pattern<E>,
            E: Error<P::Input>,
    {
        struct OrNot<A, E>(A, PhantomData<E>);

        impl<I, E, A, X> Pattern<E> for OrNot<A, E>
            where
                E: Error<I>,
                A: Pattern<E, Input=I, Output=X>,
        {
            type Input = I;
            type Output = Option<X>;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                match self.0.parse(stream) {
                    Ok((out, fail)) => Ok((Some(out), fail)),
                    Err(fail) => Ok((None, fail)),
                }
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.cloned(), PhantomData)
            }
        }

        Parser::from_pat(OrNot(self.pat, PhantomData))
    }
}
