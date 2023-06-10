pub enum Result<T, R, E> {
    Ok(T, R),
    Err(E),
}

pub use crate::Result::{Err, Ok};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEnded {}

pub trait Input: Sized {
    type Part;

    fn take_one_part(&self) -> Result<Self::Part, Self, InputEnded>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionedString<'a> {
    pub position: Position,
    pub content: &'a str,
}

impl<'a> PositionedString<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            position: Position { line: 1, column: 1 },
            content,
        }
    }
}

impl Input for &str {
    type Part = char;

    fn take_one_part(&self) -> Result<Self::Part, Self, InputEnded> {
        let mut chars = self.chars();
        match chars.next() {
            None => Err(InputEnded {}),
            Some(c) => Ok(c, chars.as_str()),
        }
    }
}

impl Input for PositionedString<'_> {
    type Part = char;

    fn take_one_part(&self) -> Result<Self::Part, Self, InputEnded> {
        let mut chars = self.content.chars();
        match chars.next() {
            None => Err(InputEnded {}),
            Some(c) => {
                let position = match c {
                    '\n' => Position {
                        line: self.position.line + 1,
                        column: 1,
                    },
                    _ => Position {
                        line: self.position.line,
                        column: self.position.column + 1,
                    },
                };
                Ok(
                    c,
                    PositionedString {
                        content: chars.as_str(),
                        position,
                    },
                )
            }
        }
    }
}

impl<T, R, E> Result<T, R, E> {
    pub fn and<OT, OR, F: FnOnce(T, R) -> Result<OT, OR, E>>(self, f: F) -> Result<OT, OR, E> {
        match self {
            Self::Ok(result, rest) => f(result, rest),
            Self::Err(err) => Err(err),
        }
    }

    pub fn or<OE, F: FnOnce(E) -> Result<T, R, OE>>(self, f: F) -> Result<T, R, OE> {
        match self {
            Ok(result, rest) => Ok(result, rest),
            Err(err) => f(err),
        }
    }

    pub fn map_rest<OR, F: FnOnce(R) -> OR>(self, f: F) -> Result<T, OR, E> {
        match self {
            Ok(result, rest) => Ok(result, f(rest)),
            Err(err) => Err(err),
        }
    }

    pub fn map<OT, F: FnOnce(T) -> OT>(self, f: F) -> Result<OT, R, E> {
        match self {
            Ok(result, rest) => Ok(f(result), rest),
            Err(err) => Err(err),
        }
    }

    pub fn map_err<OE, F: FnOnce(E) -> OE>(self, f: F) -> Result<T, R, OE> {
        match self {
            Ok(result, rest) => Ok(result, rest),
            Err(err) => Err(f(err)),
        }
    }
}

pub enum MatchingError {
    InputEnded,
    NotMatched,
}

pub fn matching<I: Input, F: FnOnce(&I::Part) -> bool>(
    input: I,
    checker: F,
) -> Result<I::Part, I, MatchingError> {
    input
        .take_one_part()
        .or(|err| match err {
            InputEnded {} => Err(MatchingError::InputEnded),
        })
        .and(|part, rest| {
            if checker(&part) {
                Ok(part, rest)
            } else {
                Err(MatchingError::NotMatched)
            }
        })
}

pub fn collect_repeating<T, E, I: Input, C: FromIterator<T>, F: Fn(&I) -> Result<T, I, E>>(
    input: I,
    parser: F,
) -> (C, I, E) {
    struct Collector<P, I, E> {
        parser: P,
        rest: I,
        error: Option<E>,
    }

    impl<T, P: Fn(&I) -> Result<T, I, E>, I: Input, E> Iterator for Collector<P, I, E> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            match (self.parser)(&self.rest) {
                Err(err) => {
                    self.error = Some(err);
                    None
                }
                Ok(item, rest) => {
                    self.rest = rest;
                    Some(item)
                }
            }
        }
    }

    let mut collector = Collector {
        parser,
        rest: input,
        error: None,
    };

    let collection = C::from_iter(&mut collector);
    (collection, collector.rest, collector.error.unwrap())
}
