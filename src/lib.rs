pub struct Or<P1, P2> {
    p1: P1,
    p2: P2,
}

pub struct OrError<P1E, P2E> {
    first_parser_error: P1E,
    second_parser_error: P2E,
}

impl<Input, Output, P1: Parser<Input, Output = Output>, P2: Parser<Input, Output = Output>>
    Parser<Input> for Or<P1, P2>
{
    type Error = OrError<P1::Error, P2::Error>;
    type Output = Output;

    fn parse(&self, input: Input) -> ParsingResult<Input, Self::Output, Self::Error> {
        match self.p1.parse(input) {
            ParsingResult::Ok { value, rest } => ParsingResult::Ok { value, rest },
            ParsingResult::Err(err1) => match self.p2.parse(input) {
                ParsingResult::Ok { value, rest } => ParsingResult::Ok { value, rest },
                ParsingResult::Err(err2) => ParsingResult::Err(OrError {
                    first_parser_error: err1,
                    second_parser_error: err2,
                }),
            },
        }
    }
}

pub trait ParserMethods {
    fn or<P2>(self, other: P2) -> Or<Self, P2>
    where
        Self: Sized;
}

impl<T> ParserMethods for T {
    fn or<P2>(self, other: P2) -> Or<Self, P2>
    where
        Self: Sized,
    {
        Or {
            p1: self,
            p2: other,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParsingResult<Input, Output, Error> {
    Ok { value: Output, rest: Input },
    Err(Error),
}

pub enum CuttingResult<Part, Input> {
    Ok { part: Part, rest: Input },
    InputEnded,
}

pub trait Input {
    type Part;

    fn cut(self) -> CuttingResult<Self::Part, Self>
    where
        Self: Sized;
}

impl Input for &str {
    type Part = char;

    fn cut(self) -> CuttingResult<Self::Part, Self>
    where
        Self: Sized,
    {
        let mut chars = self.chars();
        match chars.next() {
            Some(c) => CuttingResult::Ok {
                part: c,
                rest: chars.as_str(),
            },
            None => CuttingResult::InputEnded,
        }
    }
}

pub trait Parser<Input> {
    type Output;
    type Error;

    fn parse(&self, input: Input) -> ParsingResult<Input, Self::Output, Self::Error>;
}

impl<Input, Output, Error, T: Fn(Input) -> ParsingResult<Input, Output, Error>> Parser<Input>
    for T
{
    type Output = Output;
    type Error = Error;

    fn parse(&self, input: Input) -> ParsingResult<Input, Self::Output, Self::Error> {
        self(input)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MatchingPartGettingResult {
    InputEnded,
    InputDoesNotMatch,
}

pub trait Matcher<Value> {
    fn check(&self, value: &Value) -> bool;
}

struct EqualityMatcher<Sample> {
    pub sample: Sample,
}

struct FunctionMatcher<F> {
    pub f: F,
}

impl<Value, F: Fn(&Value) -> bool> Matcher<Value> for FunctionMatcher<F> {
    fn check(&self, value: &Value) -> bool {
        (self.f)(value)
    }
}

impl<'a, Value: 'a, Sample: PartialEq<&'a Value>> Matcher<Value> for EqualityMatcher<Sample> {
    fn check(&self, value: &Value) -> bool {
        self.sample == value
    }
}

pub fn matching_part<I: Input, M: Matcher<I::Part>>(
    matcher: M,
) -> impl Parser<I, Output = I::Part, Error = MatchingPartGettingResult> {
    move |s: I| loop {
        match s.cut() {
            CuttingResult::Ok { part, rest } => {
                if matcher.check(&part) {
                    break ParsingResult::Ok { value: part, rest };
                } else {
                    break ParsingResult::Err(MatchingPartGettingResult::InputDoesNotMatch);
                }
            }
            CuttingResult::InputEnded => {
                break ParsingResult::Err(MatchingPartGettingResult::InputEnded)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_parser() {
        let parser = matching_part(FunctionMatcher {
            f: |c: &char| "abc".contains(*c),
        });
        assert_eq!(
            parser.parse("abcdef"),
            ParsingResult::Ok {
                value: 'a',
                rest: "bcdef",
            }
        );
        assert_eq!(
            parser.parse("bc"),
            ParsingResult::Ok {
                value: 'b',
                rest: "c",
            }
        );
        assert_eq!(
            parser.parse("c"),
            ParsingResult::Ok {
                value: 'c',
                rest: "",
            }
        );
        assert_eq!(
            parser.parse("dfffffff"),
            ParsingResult::Err(MatchingPartGettingResult::InputDoesNotMatch)
        );
        assert_eq!(
            parser.parse(""),
            ParsingResult::Err(MatchingPartGettingResult::InputEnded)
        );
    }

    #[test]
    fn test_string_parser() {
        let parser = string("abc");
        assert_eq!(parser("abcdef"), Ok(("abc", "def")));
        assert_eq!(parser("def"), Err(()));
        assert_eq!(parser("abdef"), Err(()));
    }
}
