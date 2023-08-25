# parco

A library for building parser combinators.

## Project examples

* [nxml](https://github.com/megahomyak/nxml) - really outdated code, I don't recommend looking at this, this code has bad practices
* [hzlang_parser](https://github.com/megahomyak/hzlang_parser)
* [pon (its parser, to be precise)](https://github.com/megahomyak/pon/blob/master/src/parser.rs)

## Explanation

* All sequences can be split into the first element and the rest if there is at least one element
* `Input::take_one_part()` is exactly the function to split sequences
* Out of the box, only `&str` is supported. It splits into a `char` and the rest, `&str`
* Implement `Input` for types you want to work with
* The "combinator" part of the library is represented using the `parco::Result` type
* `parco::Result` can represent three states of a result of parsing: something was parsed (`Ok(output, rest)`); the input is not suitable for the current parser (`Err`); something is very wrong with the input, no parser can help (`Fatal(error)`)
* `parco::Result` should be returned from every parser
* Parsers are functions and should be either parsing small things or be combined from smaller parsers
* For example, you can make a parser that parses the word "Hello", then a parser that parses any amount of whitespace, then a parser that parses the word "world", then you can combine all these parsers into a "Hello world" parser
* `parco::Result::and` allows you to execute the next parser with the contents of the current result
* `parco::Result::or` allows you to execute any parser when the current one failed (with `Err`, `Fatal` is indeed fatal and nothing can undo it)
* `parco::Result::map` allows you to change the output of a parser to something yours (for example, if you want to add "1" to a parser's output value "123")
* `one_part()` will either return the first part of the input or fail with an `Err`
* `one_matching_part()` is just like `one_part()`, but it also fails (with `Err`) when the given predicate is not evaluating to "true"
* `collect_repeating()` will allow you to collect the results from repeating a parser on the same (shrinking) input string
* Check "Project examples" if you want to look at some neat examples

## Tips and tricks

* Write small parsers and combine them into bigger ones
* Rust-only advice: you can write a module with the name of the structure you need to parse and put a `parse` function inside, which will allow you to put tests and auxiliary functions for the parser inside the module
* Do not parse whitespace at the beginning of the input in a parser, since when you run several parsers on the same input, you can exclude whitespace beforehand. Just `Err` when there's something that's not immediately what you want to parse
