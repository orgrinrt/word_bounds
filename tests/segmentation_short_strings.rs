#[cfg(test)]
mod tests {
    use word_bounds::impls::charwalk::Charwalk;
    #[cfg(any(feature = "use_fancy_regex", feature = "benchmark"))]
    use word_bounds::impls::fancy_regex::FancyRegex;
    #[cfg(any(feature = "use_regex", feature = "benchmark"))]
    use word_bounds::impls::regex::Regex;
    use word_bounds::resolver::WordBoundResolver;
    use word_bounds::rules::DefaultRules;

    // FOR DEFAULT RULES
    const TEST_DEFAULT: &[(&str, &[&str])] = &[
        (
            "This_is_SomeRandom_Text-to-split2",
            &["this", "is", "some", "random", "text", "to", "split", "2"],
        ),
        ("_PrependedUnderscore", &["_", "prepended", "underscore"]),
        ("AppendedUnderscore_", &["appended", "underscore", "_"]),
        ("UPPERCASELETTERS", &["uppercaseletters"]),
        ("lowercaseletters", &["lowercaseletters"]),
        ("CamelCase", &["camel", "case"]),
        ("snake_case", &["snake", "case"]),
        ("kebab-case", &["kebab", "case"]),
        (
            "thisExampleHasIDELikeACRONYMS",
            &["this", "example", "has", "ide", "like", "acronyms"],
        ),
        ("WordWithNumbers123", &["word", "with", "numbers", "123"]),
        ("Short1", &["short", "1"]),
        ("number123456", &["number", "123456"]),
        ("someHTML", &["some", "html"]),
        ("JSONResponse", &["json", "response"]),
        (
            "WithSpecial-_*Characters",
            &["with", "special", "*", "characters"],
        ),
        ("hashtag#rust", &["hashtag", "#rust"]),
        // MORE COMPLICATED MIXTURES
        (
            "+This_is_SomeRandom%Text#to-split2",
            &["+", "this", "is", "some", "random%", "text", "#to", "split", "2"],
        ),
        (
            "We should definitely also test spaces, and see percents % and # hashtags with spaces",
            &[
                "we",
                "should",
                "definitely",
                "also",
                "test",
                "spaces",
                "and",
                "see",
                "percents",
                "%",
                "and",
                "#",
                "hashtags",
                "with",
                "spaces",
            ],
        ),
    ];

    #[test]
    #[cfg(any(feature = "use_fancy_regex", feature = "benchmark"))]
    fn test_word_bounds_fancy_regex() {
        for (input, expected) in TEST_DEFAULT {
            assert_eq!(
                WordBoundResolver::<FancyRegex, DefaultRules>::resolve(input),
                *expected
            );
        }
    }

    #[test]
    #[cfg(any(feature = "use_regex", feature = "benchmark"))]
    fn test_word_bounds_regex() {
        for (input, expected) in TEST_DEFAULT {
            assert_eq!(
                WordBoundResolver::<Regex, DefaultRules>::resolve(input),
                *expected
            );
        }
    }

    #[test]
    fn test_word_bounds_charwalk() {
        for (input, expected) in TEST_DEFAULT {
            assert_eq!(
                WordBoundResolver::<Charwalk, DefaultRules>::resolve(input),
                *expected
            );
        }
    }
}
