# parco

A library for building parser combinators.

## Project examples

* [nxml](https://github.com/megahomyak/nxml) - really outdated code, I don't recommend looking at this, this code has bad practices
* [hzlang_parser](https://github.com/megahomyak/hzlang_parser)
* [pon (its parser, to be precise)](https://github.com/megahomyak/pon/blob/master/src/parser.rs)

## Explanation

* All sequences can be split into the first element and the rest if there is at least one element:

    ```
    "abc" -> 'a', "bc"
    ```

* `Input::take_one_part()` is exactly the function to split sequences:

    ```
    Input::take_one_part("abc") -> Some(('a', "bc"))
    Input::take_one_part("") -> None
    ```

* Out of the box, only `&str` and `parco::PositionedString` are supported. You can split a `char` off of both of them
* You can create a `parco::PositionedString` `::from()` a `&str`:

    ```
    let positioned_string: PositionedString = "abc".into();
    ```

* Implement `Input` for types you want to work with:

    ```
    impl Input for MyType {
        type Part = MyPartType;

        fn take_one_part(&self) -> Option<(Self::Part, Self)> {
            ...
        }
    }
    ```

* The "combinator" part of the library is represented using the `parco::Result` type
* `parco::Result` can represent three states of a result of parsing: something was parsed (`Ok(output, rest)`); the input is not suitable for the current parser (`Err`); something is very wrong with the input, no parser can help (`Fatal(error)`):

    ```
    parse_string(r#"abc"#) -> Err
    parse_string(r#""hello" blah"#) -> Ok(String::from("hello"), " blah")
    parse_string(r#""\!""#) -> Fatal(Error::InvalidEscapeSequence { ... })
    ```

* `parco::Result` should be returned from every parser
* Parsers are functions and should be either parsing small things or be combined from smaller parsers
* For example, you can make a parser that parses the word "Hello", then a parser that parses any amount of whitespace, then a parser that parses the word "world", then you can combine all these parsers into a "Hello world" parser
* `parco::Result::map` allows you to change the output of a parser to something yours (for example, if you want to add "1" to a parser's output value "123")
* `parco::Result::and` allows you to execute the next parser with the contents of the current result:

    ```
    fn two_parts(rest: &str) -> parco::Result<(char, char), &str, ()> {
        parco::one_part(rest).and(|first, rest| parco::one_part(rest).map(|second| (first, second)))
    }
    ```

* `parco::Result::or` allows you to execute any parser when the current one failed (with `Err`, `Fatal` is indeed fatal and nothing can undo it):

    ```
    fn parse_declaration(rest: &str) -> parco::Result<Declaration, &str, Error> {
        parse_function_declaration(rest)
            .map(|function_declaration| Declaration::Function(function_declaration))
            .or(|| {
                parse_variable_declaration(rest)
                    .map(|variable_declaration| Declaration::Variable(variable_declaration))
            })
    }
    ```

* `one_part()` will either return the first part of the input or fail with an `Err`:

    ```
    parco::one_part("") -> Err
    parco::one_part("a") -> Ok('a', "")
    parco::one_part("abc") -> Ok('a', "bc")
    ```

* `one_matching_part()` is just like `one_part()`, but it also fails (with `Err`) when the given predicate is not evaluating to "true":

    ```
    parco::one_matching_part("", |c| *c == 'a') -> Err
    parco::one_matching_part("b", |c| *c == 'a') -> Err
    parco::one_matching_part("bbb", |c| *c == 'a') -> Err
    parco::one_matching_part("a", |c| *c == 'a') -> Ok(('a', ""))
    parco::one_matching_part("abc", |c| *c == 'a') -> Ok(('a', "bc"))
    ```

* `collect_repeating()` will allow you to collect the results from repeating a parser on the same (shrinking) input string:

    ```
    // Collecting a list of letters "a"
    parco::collect_repeating(
        Vec::new(),
        "aaabbb",
        |rest| parco::one_matching_part(*rest, |c| *c == 'a')
    )
    ```

* `collect_repeating()` returns `parco::CollResult` which only has `Ok(collection, rest)` and `Fatal(error)`, but you can turn it into a `parco::Result` using `.norm()`
* Check "Project examples" if you want to look at some neat examples

## Tips and tricks

* Write small parsers and combine them into bigger ones
* Rust-only advice: you can write a module with the name of the structure you need to parse and put a `parse` function inside, which will allow you to put tests and auxiliary functions for the parser inside the module
* Do not parse whitespace at the beginning of the input in a parser, since when you run several parsers on the same input, you can exclude whitespace beforehand. Just `Err` when there's something that's not immediately what you want to parse
* Create only one "Error" type for your entire parser and only use it in `Fatal()`
* Create an alias for `parco::Result`, thus freezing the Fatal type and the Rest type and only leaving the Output for explicit specification
