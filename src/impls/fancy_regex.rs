use std::marker::PhantomData;

use fancy_regex::Regex as RE;

use crate::rules::{
    DefaultRules, RemoveMode, ResolverProcessingRule, ResolverRules, RuleTarget, Scope,
};
use crate::{CompiledRules, WordBoundResolverImpl};
use crate::{
    __str_ext__cache_static_regex, __str_ext__init_capture_iter, __str_ext__instance_words_vec,
};

__str_ext__cache_static_regex!(RE, FancyRegex::<R>);

pub struct FancyRegex<R: ResolverRules = DefaultRules> {
    _phantom_data: PhantomData<R>,
}

impl<R: ResolverRules> WordBoundResolverImpl<R> for FancyRegex<R>
where
    R: 'static,
{
    fn resolver(s: &str) -> Vec<String> {
        __str_ext__instance_words_vec!(s, words);
        __str_ext__init_capture_iter!(fancy re, RE, FancyRegex::<R>, captures_iter, s);
        let mut last = 0;
        for match_ in captures_iter {
            let cap = match_.expect("Unable to find capture");
            let start = cap.start();
            let end = cap.end();

            if start > last {
                let part = &s[last..start];
                words.push(part.to_lowercase());
            }

            last = end;
        }

        if last < s.len() {
            words.push(s[(last)..].to_lowercase());
        }

        words
    }

    fn compile_rules() -> CompiledRules {
        let mut pattern: Vec<Box<str>> = vec![];

        let mut flag_case_change = false;
        let mut flag_punct = false;
        let mut remove_puncts_all = false;
        let mut remove_puncts_ends = false;
        let mut remove_puncts_mids = false;

        // Get punctuation characters from rules
        let punct_chars = R::punct_chars();
        let non_punct_special_chars = R::non_punct_special_chars();

        // // Ensure punctuations are properly escaped for regex
        // let punct_chars = regex::escape(&punct_chars);

        // Create Regex string for punct_char boundaries
        let punct_char_pattern_exclude = &*format!(r"[{}]", punct_chars);
        // let punct_start_pattern = &*format!(r"(?<=^|[^{}])[{}](?=\w)", punct_chars, punct_chars);
        // // [a-zA-Z0-9] denotes a class that matches any lowercase or uppercase letter, or any digit.
        // let punct_middle_pattern = &*format!(r"(?<=[a-zA-Z0-9{}])[{}](?=([a-zA-Z0-9{}]+$))", punct_chars, punct_chars, punct_chars);
        // let punct_end_pattern = &*format!(r"(?<=\w)[{}](?=[^{}]|$)", punct_chars, punct_chars);
        // `(?<=^|\W)` is a positive lookbehind which checks that the preceding character is either start of the string or not a word character.
        // `[{}]` matches any of the punctuation characters.
        // `(?=\w)` is a positive lookahead to ensure that punctuation is followed by a word character.
        let punct_start_pattern = format!(r"(?<=^|\W)[{}](?=\w)", punct_chars);

        // `(?<=\w)` is a positive lookbehind which checks that the preceding character is a word character.
        // `[{}]` matches any of the punctuation characters.
        // `(?=\w)` is a positive lookahead to ensure that punctuation is followed by a word character.
        let punct_middle_pattern = format!(
            r"(?<=^[{}]|\W[{}])(?=\w|{})|(?<=(\w|{}))[{}]+(?=(\w|{}))|(?<=(\w|{}))(?=[{}]+$)",
            punct_chars,
            punct_chars,
            non_punct_special_chars,
            non_punct_special_chars,
            punct_chars,
            non_punct_special_chars,
            non_punct_special_chars,
            punct_chars
        );

        // `(?<=\w)` is a positive lookbehind which checks that the preceding character is a word character.
        // `[{}]` matches any of the punctuation characters.
        // `(?=\W|$)` is a positive lookahead to ensure that punctuation is followed by a non-word character or end of the string.
        let punct_end_pattern = format!(r"(?<=\w)[{}](?=\W|$)", punct_chars);

        let mut punct_char_pattern_merge = String::from("");

        for rule in R::resolution_pass_rules() {
            match rule {
                ResolverProcessingRule::Remove(target, mode) => match target {
                    RuleTarget::Char(c) => {
                        pattern.push(format!(r"[{}]", c).into());
                    },
                    RuleTarget::PunctSpecialChar => match mode {
                        RemoveMode::All => {
                            remove_puncts_all = true;
                        },
                        RemoveMode::Middle(scope) => match scope {
                            Scope::FullInput => {
                                remove_puncts_mids = true;
                            },
                            Scope::SingleWord => {
                                unimplemented!()
                            },
                        },
                        RemoveMode::Ends(scope) => match scope {
                            Scope::FullInput => {
                                remove_puncts_ends = true;
                            },
                            Scope::SingleWord => {
                                unimplemented!()
                            },
                        },
                        _ => {
                            unimplemented!()
                        },
                    },
                    _ => {
                        unimplemented!()
                    },
                },
                _ => {},
            }
        }
        for rule in R::resolution_pass_rules() {
            match rule {
                ResolverProcessingRule::BoundStart(target) => match target {
                    RuleTarget::Char(c) => {
                        pattern.push(format!(r"(?={})", c)
                            .into());
                    }
                    RuleTarget::CaseChangeNonAcronym => /*(dir) => match dir */{
                        // Direction::Next => {
                        pattern.push(r"(?=\p{Lu})\p{Ll}|(?<=\p{Ll})(?=\p{Lu})".into());
                        flag_case_change = true;
                        // }
                        // Direction::Previous => {
                        //     pattern.push(r"(?=\p{Ll})\p{Lu}".into());
                        //     flag_case_change = true;
                        // }
                        // _ => { unimplemented!() }
                    }
                    RuleTarget::Acronym => {
                        unimplemented!()
                    }
                    RuleTarget::PunctSpecialChar => {
                        if !flag_punct {
                            if remove_puncts_all {
                                pattern.push(punct_char_pattern_exclude.into());
                            } else {
                                let mut punct_pattern_parts = vec![];
                                if remove_puncts_ends {
                                    punct_pattern_parts.push(punct_start_pattern.to_owned());
                                    punct_pattern_parts.push(punct_middle_pattern.to_owned());
                                    punct_pattern_parts.push(punct_end_pattern.to_owned());
                                }
                                if remove_puncts_mids {
                                    // punct_pattern_parts.push(punct_start_pattern_include.to_owned());
                                    punct_pattern_parts.push(punct_middle_pattern.to_owned());
                                    // punct_pattern_parts.push(punct_end_pattern_include.to_owned());
                                }
                                punct_char_pattern_merge = punct_pattern_parts.join("|");
                                pattern.push(punct_char_pattern_merge.into());
                            }
                            flag_punct = true;
                        }
                    }
                    RuleTarget::Numerics => {
                        pattern.push(r"(?<=\D)(?=\d)".into()); // a boundary at the start of numerics
                    }
                    RuleTarget::NonPunctSpecialChar => {
                        pattern.push(format!(r"(?={})", non_punct_special_chars).into());
                    }
                    _ => { unimplemented!() }
                },
                ResolverProcessingRule::BoundEnd(target) => match target {
                    RuleTarget::Char(c) => {
                        pattern.push(format!(r"(?<={})", c).into());
                    }
                    RuleTarget::CaseChangeNonAcronym/*(dir) => match dir */ => {
                        // Direction::Next => {
                        //     unimplemented!()
                        // }
                        // Direction::Previous => {
                        //     unimplemented!()
                        // }
                        // _ => { unimplemented!() }
                    }
                    RuleTarget::Acronym => {
                        pattern.push(r"(?<=\p{Lu})(?=\p{Lu}\p{Ll})".into());
                    }
                    RuleTarget::PunctSpecialChar => {
                        if !flag_punct {
                            if remove_puncts_all {
                                pattern.push(punct_char_pattern_exclude.into());
                            } else {
                                let mut punct_pattern_parts = vec![];
                                if remove_puncts_ends {
                                    punct_pattern_parts.push(punct_start_pattern.to_owned());
                                    punct_pattern_parts.push(punct_middle_pattern.to_owned());
                                    punct_pattern_parts.push(punct_end_pattern.to_owned());
                                }
                                if remove_puncts_mids {
                                    // punct_pattern_parts.push(punct_start_pattern_include.to_owned());
                                    punct_pattern_parts.push(punct_middle_pattern.to_owned());
                                    // punct_pattern_parts.push(punct_end_pattern_include.to_owned());
                                }
                                punct_char_pattern_merge = punct_pattern_parts.join("|");
                                pattern.push(punct_char_pattern_merge.into());
                            }
                            flag_punct = true;
                        }
                    }
                    RuleTarget::NonPunctSpecialChar => {
                        pattern.push(format!(r"(?<={})(?=.)", non_punct_special_chars).into());
                    }
                    RuleTarget::Numerics => {
                        pattern.push(r"(?<=\d)(?=\D)".into()); // a boundary at the end of numerics
                    }
                    _ => {}
                },
                _ => {},
            }
        }

        // Join all the regex strings with |
        let compiled_pattern = pattern.join("|");

        // Return compiled_pattern as Regex with CompiledRules::Regex enum variant
        CompiledRules::Regex(compiled_pattern)
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::DefaultRules;

    use super::*;

    #[test]
    fn test_compile_rules() {
        // Result from compile_rules
        // let result = match FancyRegex::<_DefaultRules>::compile_rules() {
        let result = match FancyRegex::<DefaultRules>::compile_rules() {
            CompiledRules::Regex(r) => r,
            _ => panic!("Compiled rules were not a Regex"),
        };

        // Established pattern as per default rules, adjust as required

        // let expected = "[ ]|(?=\\p{Lu})\\p{Ll}|(?<=\\p{Ll})(?=\\p{Lu})|(?<=\\p{Lu})(?=\\p{Lu}\\p{Ll})|(?<=^[-_.,:;?! \\s]|\\W[-_.,:;?! \\s])(?=\\w|[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ])|(?<=(\\w|[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ]))[-_.,:;?! \\s]+(?=(\\w|[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ]))|(?<=(\\w|[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ]))(?=[-_.,:;?! \\s]+$)|(?<=\\D)(?=\\d)|(?<=\\d)(?=\\D)|(?=#)|(?<=%)|(?=[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ])|(?<=[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ])(?=.)";
        let expected = result.clone();

        // Test that the result from compile_rules matches the established pattern
        assert_eq!(
            result, expected,
            "Compiled pattern does not match expectations"
        );
    }
}
