pub trait Input {
    type Part;

    fn take_one_part(&self) -> Option<(Self::Part, Self)>
    where
        Self: Sized;
}

impl Input for &str {
    type Part = char;

    fn take_one_part(&self) -> Option<(Self::Part, Self)> {
        let mut chars = self.chars();
        chars.next().map(|c| (c, chars.as_str()))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Position {
    pub row: usize,
    /// Column
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionedString<'s> {
    pub content: &'s str,
    pub position: Position,
}

impl<'a> PositionedString<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            position: Position { row: 1, column: 1 },
        }
    }
}

impl<'s> Input for PositionedString<'s> {
    type Part = char;

    fn take_one_part(&self) -> Option<(Self::Part, Self)> {
        let mut chars = self.content.chars();
        chars.next().map(|c| {
            (
                c,
                Self {
                    content: chars.as_str(),
                    position: if c == '\n' {
                        Position {
                            row: self.position.row + 1,
                            column: 1,
                        }
                    } else {
                        Position {
                            row: self.position.row,
                            column: self.position.column + 1,
                        }
                    },
                },
            )
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Result<T, I, F> {
    /// Parsing completed successfully
    Ok(T, I),
    /// Recoverable error meaning "input cannot be parsed with the current parser"
    Err,
    /// Unrecoverable error meaning "input cannot be parsed with any parser"
    Fatal(F),
}

use crate::Result::{Err, Fatal, Ok};

impl<T, I, F> Result<T, I, F> {
    pub fn and<OT, OI>(self, f: impl FnOnce(T, I) -> Result<OT, OI, F>) -> Result<OT, OI, F> {
        match self {
            Ok(result, rest) => f(result, rest),
            Err => Err,
            Fatal(e) => Fatal(e),
        }
    }

    pub fn or(self, f: impl FnOnce() -> Self) -> Self {
        match self {
            Ok(result, rest) => Ok(result, rest),
            Err => f(),
            Fatal(e) => Fatal(e),
        }
    }

    pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Result<O, I, F> {
        match self {
            Ok(result, rest) => Ok(f(result), rest),
            Err => Err,
            Fatal(e) => Fatal(e),
        }
    }
}

pub fn one_part<I: Input, F>(input: I) -> Result<I::Part, I, F> {
    input
        .take_one_part()
        .map_or(Err, |(part, rest)| Ok(part, rest))
}

pub fn one_matching_part<I: Input, F>(
    input: I,
    f: impl FnOnce(&I::Part) -> bool,
) -> Result<I::Part, I, F> {
    one_part(input).and(|part, rest| if f(&part) { Ok(part, rest) } else { Err })
}

pub fn collect_repeating<T, I, F, P: Fn(&I) -> Result<T, I, F>, C: Extend<T>>(
    mut collection: C,
    input: I,
    parser: P,
) -> Result<C, I, F> {
    struct Collector<P, I, F> {
        parser: P,
        rest: I,
        fatal_error: Option<F>,
    }

    impl<T, I, P: Fn(&I) -> Result<T, I, F>, F> Iterator for Collector<P, I, F> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            match (self.parser)(&self.rest) {
                Err => None,
                Fatal(err) => {
                    self.fatal_error = Some(err);
                    None
                }
                Ok(result, rest) => {
                    self.rest = rest;
                    Some(result)
                }
            }
        }
    }

    let mut collector = Collector {
        fatal_error: None,
        rest: input,
        parser,
    };
    collection.extend(&mut collector);
    match collector.fatal_error {
        None => Ok(collection, collector.rest),
        Some(err) => Fatal(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taking_one_part() {
        assert_eq!(one_part::<_, ()>("abc"), Ok('a', "bc"));

        assert_eq!(one_part::<_, ()>(""), Err);
    }

    #[test]
    fn test_taking_one_matching_part() {
        assert_eq!(
            one_matching_part::<_, ()>("123", |c| c.is_numeric()),
            Ok('1', "23")
        );

        assert_eq!(one_matching_part::<_, ()>("_?!", |c| c.is_numeric()), Err);

        assert_eq!(one_matching_part::<_, ()>("", |_c| true), Err);
    }

    #[test]
    fn test_collecting() {
        let result = collect_repeating(Vec::new(), "123abc", |input| {
            one_matching_part::<_, ()>(input, |c| c.is_numeric())
        });

        assert_eq!(result, Ok(vec!['1', '2', '3'], "abc"));

        let result = collect_repeating(Vec::new(), "abc", |input| {
            one_matching_part::<_, ()>(input, |c| c.is_numeric())
        });

        assert_eq!(result, Ok(vec![], "abc"));

        let result = collect_repeating(Vec::new(), "123", |input| {
            one_matching_part::<_, ()>(input, |c| c.is_numeric())
        });

        assert_eq!(result, Ok(vec!['1', '2', '3'], ""));

        let result = collect_repeating::<(), _, _, _, _>(Vec::new(), "", |_input| Fatal(()));

        assert_eq!(result, Fatal(()));
    }

    #[test]
    fn test_sequential_parsing() {
        let input = "12345";

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1').and(|c1, input| one_matching_part(
                input,
                |c| *c == '2'
            )
            .map(|c2| [c1, c2].iter().collect::<String>())),
            Ok(String::from("12"), "345")
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .and(|_c, input| one_matching_part(input, |c| *c == '1')),
            Err,
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1')
                .and(|_c, input| one_matching_part(input, |c| *c == 'b')),
            Err,
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .and(|_c, input| one_matching_part(input, |c| *c == 'b')),
            Err,
        );
    }

    #[test]
    fn test_alternative_parsing() {
        let input = "12345";

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .or(|| one_matching_part(input, |c| *c == '1')),
            Ok('1', "2345")
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == 'a')
                .or(|| one_matching_part(input, |c| *c == 'b')),
            Err,
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1')
                .or(|| one_matching_part(input, |c| *c == 'b')),
            Ok('1', "2345")
        );

        assert_eq!(
            one_matching_part::<_, ()>(input, |c| *c == '1')
                .map(|_c| 'a')
                .or(|| one_matching_part(input, |c| *c == '1')),
            Ok('a', "2345")
        );
    }

    #[test]
    fn test_output_mapping() {
        assert_eq!(
            one_part::<_, ()>("1").map(|_c| String::from("Hello!")),
            Ok(String::from("Hello!"), "")
        );

        assert_eq!(one_part::<_, ()>("").map(|_c| String::from("Hello!")), Err);
    }

    #[test]
    fn test_position_tracking() {
        assert_eq!(PositionedString::new("").position, Position { row: 1, column: 1 });

        assert_eq!(
            one_part::<_, ()>(PositionedString::new("1")),
            Ok(
                '1',
                PositionedString {
                    position: Position { row: 1, column: 2 },
                    content: ""
                }
            )
        );

        assert_eq!(
            one_part::<_, ()>(PositionedString::new("a\n")).and(|_c, rest| one_part(rest)),
            Ok(
                '\n',
                PositionedString {
                    position: Position { row: 2, column: 1 },
                    content: ""
                }
            )
        );
    }
}
