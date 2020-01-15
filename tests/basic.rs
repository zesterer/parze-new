use parze::prelude::*;

#[test]
fn a() {
    // Any

    let p = any::<_, DefaultError<char>>();

    assert_eq!(p.parse(vec!['!']), Ok('!'));
    assert!(p.parse(vec![]).is_err());

    // End

    let p = end::<_, DefaultError<char>>();

    assert_eq!(p.parse(vec![]), Ok(()));
    assert!(p.parse(vec!['!']).is_err());

    // Map

    let p = any::<_, DefaultError<char>>().map(|_| '?');

    assert_eq!(p.parse(vec!['!']), Ok('?'));

    // Just

    let p = just::<_, _, DefaultError<_>>('!');

    assert!(p.parse(vec!['?']).is_err());

    // Then

    let p = just::<_, _, DefaultError<_>>('!').then(just('?'));

    assert_eq!(p.parse(vec!['!', '?']), Ok(('!', '?')));
    assert!(p.parse(vec!['!', '!']).is_err());
    assert!(p.parse(vec!['?', '?']).is_err());

    // Or

    let p = just::<_, _, DefaultError<_>>('!').or(just('?'));

    assert_eq!(p.parse(vec!['!']), Ok('!'));
    assert_eq!(p.parse(vec!['?']), Ok('?'));
    assert!(p.parse(vec!['@']).is_err());

    // Repeated

    let p = just::<_, _, DefaultError<_>>('!').repeated();

    assert_eq!(p.parse(vec!['!', '!', '!']), Ok(vec!['!', '!', '!']));
    assert_eq!(p.parse(vec!['!', '!', '?']), Ok(vec!['!', '!']));
    assert_eq!(p.parse(vec!['?']), Ok(vec![]));
    assert_eq!(p.parse(vec![]), Ok(vec![]));

    // OnceOrMore

    let p = just::<_, _, DefaultError<_>>('!').once_or_more();

    assert_eq!(p.parse(vec!['!', '!', '!']), Ok(vec!['!', '!', '!']));
    assert_eq!(p.parse(vec!['!', '!', '?']), Ok(vec!['!', '!']));
    assert_eq!(p.parse(vec!['!', '?']), Ok(vec!['!']));
    assert!(p.parse(vec!['?']).is_err());
    assert!(p.parse(vec![]).is_err());

    // OrNot

    let p = just::<_, _, DefaultError<_>>('!').or_not();

    assert_eq!(p.parse(vec!['!']), Ok(Some('!')));
    assert_eq!(p.parse(vec!['?']), Ok(None));
    assert_eq!(p.parse(vec![]), Ok(None));
}

#[test]
fn bf() {
    #[derive(Clone, Debug, PartialEq)]
    enum Instr {
        Add,
        Sub,
        Left,
        Right,
        In,
        Out,
        Loop(Vec<Instr>),
    }

    let bf = recursive(|bf| {
            just::<_, _, DefaultError<_>>('+').to(Instr::Add)
        .or(just('-').to(Instr::Sub))
        .or(just('<').to(Instr::Left))
        .or(just('>').to(Instr::Right))
        .or(just(',').to(Instr::In))
        .or(just('.').to(Instr::Out))
        .or(just('[').then(bf.link()).then(just(']')).map(|((_, i), _)| Instr::Loop(i)))
        .repeated()
    });

    assert_eq!(
        bf.parse("++--[->++<].".chars()),
        Ok(vec![
            Instr::Add,
            Instr::Add,
            Instr::Sub,
            Instr::Sub,
            Instr::Loop(vec![
                Instr::Sub,
                Instr::Right,
                Instr::Add,
                Instr::Add,
                Instr::Left,
            ]),
            Instr::Out,
        ]),
    );
}
