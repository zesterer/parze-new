use std::{
    rc::Rc,
    cell::RefCell,
    marker::PhantomData,
};
use crate::{
    Parser,
    Pattern,
    Error,
    Stream,
    ParseResult,
};

pub struct Declaration<E, I, O> {
    parser: Rc<RefCell<Option<Box<dyn Pattern<E, Input=I, Output=O>>>>>,
}

impl<E, I, O> Default for Declaration<E, I, O> {
    fn default() -> Self {
        Self { parser: Rc::new(RefCell::new(None)) }
    }
}

impl<E, I, O> Declaration<E, I, O>
    where E: Error<I>
{
    pub fn link(&self) -> Parser<impl Pattern<E, Input=I, Output=O>, E> {
        struct Linked<I, O, E>(Rc<RefCell<Option<Box<dyn Pattern<E, Input=I, Output=O>>>>>, PhantomData<(I, O)>);

        impl<I, O, E> Pattern<E> for Linked<I, O, E>
            where E: Error<I>
        {
            type Input = I;
            type Output = O;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                self.0.borrow().as_ref().as_ref().unwrap().parse(stream)
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.clone(), PhantomData)
            }
        }

        Parser::from_pat(Linked(self.parser.clone(), PhantomData))
    }

    pub fn define(self, parser: Parser<impl Pattern<E, Input=I, Output=O> + 'static, E>) -> Parser<impl Pattern<E, Input=I, Output=O>, E> {
        struct Defined<I, O, E>(Rc<RefCell<Option<Box<dyn Pattern<E, Input=I, Output=O>>>>>);

        impl<I, O, E> Pattern<E> for Defined<I, O, E>
            where E: Error<I>
        {
            type Input = I;
            type Output = O;

            fn parse(&self, stream: &mut Stream<Self::Input>) -> ParseResult<Self::Output, E> {
                self.0.borrow().as_ref().as_ref().unwrap().parse(stream)
            }

            fn cloned(&self) -> Self where Self: Sized {
                Self(self.0.clone())
            }
        }

        *self.parser.borrow_mut() = Some(Box::new(parser.pat));

        Parser::from_pat(Defined(self.parser))
    }
}

pub fn declare<E, I, O>() -> Declaration<E, I, O>
    where E: Error<I>
{
    Declaration::default()
}

pub fn recursive<E, I, O, P>(f: impl FnOnce(&Declaration<E, I, O>) -> Parser<P, E>) -> Parser<impl Pattern<E, Input=I, Output=O>, E>
    where
        E: Error<I>,
        P: Pattern<E, Input=I, Output=O> + 'static,
{
    let declaration = declare();
    let parser = f(&declaration);
    declaration.define(parser)
}

