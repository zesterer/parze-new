use crate::Stream;

pub fn attempt<'a, T, R, F, E>(tokens: &mut Stream<T>, f: F) -> Result<R, E>
    where F: FnOnce(&mut Stream<T>) -> Result<R, E>,
{
    let mut tokens2 = tokens.clone();
    let tok = f(&mut tokens2)?;
    *tokens = tokens2;
    Ok(tok)
}
