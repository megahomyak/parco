#[derive(Debug, PartialEq, Eq)]
pub struct Rest<T>(pub T);

pub trait Input {
    type Part;

    fn take_one_part(&self) -> Option<(Self::Part, Rest<Self>)>
    where
        Self: Sized;
}

impl Input for &str {
    type Part = char;

    fn take_one_part(&self) -> Option<(Self::Part, Rest<Self>)> {
        let mut chars = self.chars();
        chars.next().map(|c| (c, Rest(chars.as_str())))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub row: usize,
    /// Column
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionedString<'s> {
    src: &'s str,
    pos: Position,
}

impl<'s> PositionedString<'s> {
    pub fn src(&self) -> &str {
        self.src
    }

    pub fn pos(&self) -> Position {
        self.pos
    }
}

impl<'s> From<&'s str> for PositionedString<'s> {
    fn from(src: &'s str) -> Self {
        Self {
            src,
            pos: Position { row: 1, col: 1 },
        }
    }
}

impl<'s> Input for PositionedString<'s> {
    type Part = char;

    fn take_one_part(&self) -> Option<(Self::Part, Rest<Self>)> {
        let mut chars = self.src.chars();
        chars.next().map(|c| {
            (
                c,
                Rest(Self {
                    src: chars.as_str(),
                    pos: if c == '\n' {
                        Position {
                            row: self.pos.row + 1,
                            col: 1,
                        }
                    } else {
                        Position {
                            row: self.pos.row,
                            col: self.pos.col + 1,
                        }
                    },
                }),
            )
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Result<T, I, F> {
    /// Parsing completed successfully
    Ok((T, Rest<I>)),
    /// Recoverable error meaning "input cannot be parsed with the current parser"
    Err,
    /// Unrecoverable error meaning "input cannot be parsed with any parser"
    Fatal(F),
}

use crate::Result::{Err, Fatal, Ok};

impl<T, I, F> Result<T, I, F> {
    pub fn and<O>(self, f: impl FnOnce((T, Rest<I>)) -> Result<O, I, F>) -> Result<O, I, F> {
        match self {
            Self::Ok(success) => f(success),
            Self::Err => Err,
            Self::Fatal(e) => Fatal(e),
        }
    }

    pub fn or(self, f: impl FnOnce() -> Self) -> Self {
        if let Self::Err = self {
            f()
        } else {
            self
        }
    }

    pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Result<O, I, F> {
        match self {
            Self::Ok((value, rest)) => Ok((f(value), rest)),
            Self::Err => Err,
            Self::Fatal(e) => Fatal(e),
        }
    }
}

pub fn one_part<I: Input, F>(input: I) -> Result<I::Part, I, F> {
    input
        .take_one_part()
        .map_or(Err, |(part, rest)| Ok((part, rest)))
}

pub fn one_matching_part<I: Input, F>(
    input: I,
    f: impl FnOnce(&I::Part) -> bool,
) -> Result<I::Part, I, F> {
    one_part(input).and(|(part, rest)| if f(&part) { Ok((part, rest)) } else { Err })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taking_one_part() {
        assert_eq!(one_part::<_, ()>("abc"), Ok(('a', Rest("bc"))));

        assert_eq!(one_part::<_, ()>(""), Err);
    }

    #[test]
    fn test_taking_one_matching_part() {
        assert_eq!(
            one_matching_part::<_, ()>("123", |c| c.is_numeric()),
            Ok(('1', Rest("23")))
        );

        assert_eq!(one_matching_part::<_, ()>("_?!", |c| c.is_numeric()), Err);

        assert_eq!(one_matching_part::<_, ()>("", |_c| true), Err);
    }

    #[test]
    fn test_sequential_parsing() {
        let input = "12345";

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1').and(|(c1, input)| one_matching_part(
                input.0,
                |c| *c == '2'
            )
            .map(|c2| [c1, c2].iter().collect::<String>())),
            Ok((String::from("12"), Rest("345")))
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .and(|(_c, input)| one_matching_part(input.0, |c| *c == '1')),
            Err,
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1')
                .and(|(_c, input)| one_matching_part(input.0, |c| *c == 'b')),
            Err,
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .and(|(_c, input)| one_matching_part(input.0, |c| *c == 'b')),
            Err,
        );
    }

    #[test]
    fn test_alternative_parsing() {
        let input = "12345";

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .or(|| one_matching_part(input, |c| *c == '1')),
            Ok(('1', Rest("2345")))
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .or(|| one_matching_part(input, |c| *c == 'b')),
            Err,
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1')
                .or(|| one_matching_part(input, |c| *c == 'b')),
            Ok(('1', Rest("2345")))
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1')
                .map(|_c| 'a')
                .or(|| one_matching_part(input, |c| *c == '1')),
            Ok(('a', Rest("2345")))
        );
    }

    #[test]
    fn test_output_mapping() {
        assert_eq!(
            one_part::<_, ()>("1").map(|_c| String::from("Hello!")),
            Ok((String::from("Hello!"), Rest("")))
        );

        assert_eq!(one_part::<_, ()>("").map(|_c| String::from("Hello!")), Err);
    }

    #[test]
    fn test_position_tracking() {
        assert_eq!(
            PositionedString::from("").pos(),
            Position { row: 1, col: 1 }
        );

        assert_eq!(
            one_part::<_, ()>(PositionedString::from("1")),
            Ok((
                '1',
                Rest(PositionedString {
                    pos: Position { row: 1, col: 2 },
                    src: ""
                })
            ))
        );

        assert_eq!(
            one_part::<_, ()>(PositionedString::from("a\n")).and(|(_c, rest)| one_part(rest.0)),
            Ok((
                '\n',
                Rest(PositionedString {
                    pos: Position { row: 2, col: 1 },
                    src: ""
                })
            ))
        );
    }
}
