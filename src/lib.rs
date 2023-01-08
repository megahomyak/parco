pub fn character(checker: impl Fn(char) -> bool) -> impl Fn(&str) -> Result<(char, &str), ()> {
    move |s| {
        let mut chars = s.chars();
        if let Some(c) = chars.next() {
            if checker(c) {
                Ok((c, chars.as_str()))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}

pub fn string<'a, 'b>(sample: &'a str) -> impl Fn(&'b str) -> Result<(&'a str, &'b str), ()> {
    move |input| {
        let mut input_characters = input.chars();
        for sample_character in sample.chars() {
            if let Some(input_character) = input_characters.next() {
                if input_character != sample_character {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }
        return Ok((sample, input_characters.as_str()));
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_parser() {
        let parser = character(|c| "abc".contains(c));
        assert_eq!(parser("abcdef"), Ok(('a', "bcdef")));
        assert_eq!(parser("bc"), Ok(('b', "c")));
        assert_eq!(parser("c"), Ok(('c', "")));
        assert_eq!(parser("dffffff"), Err(()));
        assert_eq!(parser(""), Err(()));
    }

    #[test]
    fn test_string_parser() {
        let parser = string("abc");
        assert_eq!(parser("abcdef"), Ok(("abc", "def")));
        assert_eq!(parser("def"), Err(()));
        assert_eq!(parser("abdef"), Err(()));
    }
}
