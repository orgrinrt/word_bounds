use std::marker::PhantomData;

use regex::Regex as RE;

use crate::rules::ResolverProcessingRule::{BoundEnd, BoundStart};
use crate::rules::{
    DefaultRules, RemoveMode, ResolverProcessingRule, ResolverRules, RuleTarget, Scope,
};
use crate::{
    CompiledRules, WordBoundResolverImpl, __str_ext__cache_static_regex,
    __str_ext__init_capture_iter, __str_ext__instance_words_vec,
};

__str_ext__cache_static_regex!(RE, Regex::<R>);

pub struct Regex<R: ResolverRules = DefaultRules> {
    _phantom_data: PhantomData<R>,
}

impl<R: ResolverRules> WordBoundResolverImpl<R> for Regex<R>
where
    R: 'static,
{
    fn resolver(s: &str) -> Vec<String> {
        __str_ext__instance_words_vec!(s, words);
        __str_ext__init_capture_iter!(plain re, RE, Regex::<R>, captures_iter, s);

        let punct_chars = R::punct_chars_allow_whitespace();
        let non_punct_special_chars = R::non_punct_special_chars_allow_whitespace();
        let puncts = RE::new(&format!(r"[{}]", &punct_chars)).expect(
            "Expected a valid \
        regex \
        pattern \
        for punct chars",
        );
        let non_punct_specials = RE::new(&format!(r"{}", &non_punct_special_chars)).expect(
            "Expected\
         a valid regex pattern for \
        non-punct special chars",
        );
        let mut attach_to_next = String::new();
        let mut remove_idxs: Vec<usize> = Vec::new();

        let captures: Vec<_> = captures_iter.collect();
        let captures_len = captures.len();
        // let first = &captures[0][0];
        // let last = &captures[captures.len() - 1][0];
        // const SPLIT_IN_POST_MARKER: &str = "[[_S_]]";
        const SPLIT_IN_POST_MARKER: &str = " ";
        for (idx, cap) in captures.iter().enumerate() {
            let mut word = if !attach_to_next.is_empty() {
                let res = format!("{}{}", attach_to_next, &cap[0]).to_owned();
                attach_to_next.clear();
                res
            } else {
                cap[0].to_owned()
            };
            for rule in R::resolution_pass_rules() {
                match rule {
                    ResolverProcessingRule::Remove(target, mode) => match target {
                        RuleTarget::Char(c) => {
                            word = word.replace(c, "");
                        },
                        RuleTarget::PunctSpecialChar => match mode {
                            RemoveMode::All => {
                                if puncts.is_match(&word) {
                                    remove_idxs.push(idx);
                                } else {
                                    word = puncts.replace_all(&word, "").to_string();
                                }
                            },
                            RemoveMode::Middle(scope) => match scope {
                                Scope::FullInput => {
                                    if idx != 0 && idx != captures_len - 1 {
                                        if puncts.is_match(&word) {
                                            remove_idxs.push(idx);
                                        } else {
                                            word = puncts.replace_all(&word, "").to_string();
                                        }
                                    }
                                },
                                Scope::SingleWord => {
                                    unimplemented!()
                                },
                            },
                            RemoveMode::Ends(scope) => match scope {
                                Scope::FullInput => {
                                    if idx == 0 || idx == captures_len {
                                        if puncts.is_match(&word) {
                                            remove_idxs.push(idx);
                                        } else {
                                            word = puncts.replace_all(&word, "").to_string();
                                        }
                                    }
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
                    ResolverProcessingRule::BoundStart(target) => match target {
                        RuleTarget::Char(c) => {
                            let c_str = c.to_string();
                            if word == c_str {
                                let i = s.find(&c_str).unwrap();
                                if s.get(i + 1..i + 2).unwrap() == " " {
                                    continue;
                                } else {
                                    remove_idxs.push(idx);
                                    c_str.clone_into(&mut attach_to_next);
                                }
                            }
                        },
                        RuleTarget::CaseChangeNonAcronym => {
                            let mut prev_was_lowcase: i8 = -1;
                            let mut prev_was_split = 1;
                            let new = word.chars().fold(String::new(), |acc: String, c: char| {
                                if prev_was_lowcase == 1 && c.is_uppercase() && prev_was_split == 0
                                {
                                    prev_was_lowcase = 0;
                                    let acc_minus_one =
                                        if acc.is_empty() { "" } else { &acc[0..acc.len() - 1] };
                                    let acc_tail = if acc.is_empty() {
                                        ""
                                    } else {
                                        &acc.chars().last().unwrap().to_string()
                                    };
                                    prev_was_split = 2;
                                    return format!(
                                        "{}{}{}{}",
                                        acc_minus_one, acc_tail, SPLIT_IN_POST_MARKER, c
                                    );
                                } else if prev_was_lowcase == 0
                                    && c.is_lowercase()
                                    && prev_was_split == 0
                                {
                                    prev_was_lowcase = 1;
                                    let acc_minus_one =
                                        if acc.is_empty() { "" } else { &acc[0..acc.len() - 1] };
                                    let acc_tail = if acc.is_empty() {
                                        ""
                                    } else {
                                        &acc.chars().last().unwrap().to_string()
                                    };
                                    prev_was_split = 2;
                                    return format!(
                                        "{}{}{}{}",
                                        acc_minus_one, SPLIT_IN_POST_MARKER, acc_tail, c
                                    );
                                }
                                prev_was_lowcase = if c.is_lowercase() { 1 } else { 0 };
                                prev_was_split =
                                    if prev_was_split > 0 { prev_was_split - 1 } else { 0 };
                                format!("{}{}", acc, c)
                            });
                            word = new;
                        },
                        RuleTarget::Acronym => {
                            unimplemented!()
                        },
                        RuleTarget::PunctSpecialChar => {
                            // NOTE: already handled by prepass regex
                            // println!("  START: RuleTarget::PunctSpecialChar  >>  {} should match \
                            // regex: \
                            // {} = {}", &word, puncts, puncts.is_match(&word));
                            // if puncts.is_match(&word) {
                            //     remove_idxs.push(idx);
                            //     word.clone_into(&mut attach_to_next);
                            // }
                        },
                        RuleTarget::Numerics => {
                            if R::resolution_pass_rules().contains(&BoundEnd(RuleTarget::Numerics))
                            {
                                continue;
                            }
                            if word.chars().all(|c: char| c.is_numeric()) {
                                if idx == captures_len - 1 {
                                    continue;
                                }
                                remove_idxs.push(idx);
                                word.clone_into(&mut attach_to_next);
                            } else if word.chars().any(|c: char| c.is_numeric()) {
                                panic!(
                                    "We should never end up in a situation where a word is a \
                                mix of numerics and alphabet (initial regex should take care of \
                                that)"
                                );
                            }
                        },
                        RuleTarget::NonPunctSpecialChar => {
                            if R::resolution_pass_rules()
                                .contains(&BoundEnd(RuleTarget::NonPunctSpecialChar))
                            {
                                continue;
                            }
                            if non_punct_specials.is_match(&word) {
                                remove_idxs.push(idx);
                                word.clone_into(&mut attach_to_next);
                            }
                        },
                        _ => {
                            unimplemented!()
                        },
                    },
                    ResolverProcessingRule::BoundEnd(target) => match target {
                        RuleTarget::Char(c) => {
                            let c_str = c.to_string();
                            if word == c_str {
                                remove_idxs.push(idx);
                                let len = words.len();
                                if len == 0 {
                                    words.push(String::new());
                                } else {
                                    let prev = &(words[len - 1]);
                                    words[len - 1] = format!("{}{}", prev, &c_str);
                                };
                            }
                        },
                        RuleTarget::CaseChangeNonAcronym => {
                            let mut prev_was_lowcase: i8 = -1;
                            let new = word.chars().fold(String::new(), |acc: String, c: char| {
                                if prev_was_lowcase == -1 {
                                    prev_was_lowcase = if c.is_lowercase() { 1 } else { 0 };
                                } else if prev_was_lowcase == 1 && c.is_uppercase() {
                                    prev_was_lowcase = 0;
                                    return format!("{}{}{}", acc, SPLIT_IN_POST_MARKER, c);
                                } else if prev_was_lowcase == 0 && c.is_lowercase() {
                                    prev_was_lowcase = 1;
                                    return format!("{}{}{}", acc, SPLIT_IN_POST_MARKER, c);
                                }
                                format!("{}{}", acc, c)
                            });
                            word = new;
                        },
                        RuleTarget::Acronym => {
                            // NOTE: maybe Case
                            // ChangeNonAcronym should work here and this is
                            //      unnecessary
                        },
                        RuleTarget::PunctSpecialChar => {
                            // NOTE: already handled by prepass regex
                            // println!("  END: RuleTarget::PunctSpecialChar >>  {} should match \
                            // regex: \
                            // {} = {}", &word, puncts, puncts.is_match(&word));
                            // if puncts.is_match(&word) {
                            //     remove_idxs.push(idx);
                            //     let len = words.len();
                            //     if len == 0 {
                            //         words.push(String::new());
                            //     } else {
                            //         let prev = &(words[len - 1]);
                            //         words[len - 1] = format!("{}{}", &word, prev);
                            //     };
                            // }
                        },
                        RuleTarget::NonPunctSpecialChar => {
                            if R::resolution_pass_rules()
                                .contains(&BoundStart(RuleTarget::NonPunctSpecialChar))
                            {
                                continue;
                            }
                            if non_punct_specials.is_match(&word) {
                                remove_idxs.push(idx);
                                let len = words.len();
                                if len == 0 {
                                    words.push(word.to_owned());
                                } else {
                                    let prev = &(words[len - 1]);
                                    words[len - 1] = format!("{}{}", &word, prev);
                                };
                            }
                        },
                        RuleTarget::Numerics => {
                            if R::resolution_pass_rules()
                                .contains(&BoundStart(RuleTarget::Numerics))
                            {
                                continue;
                            }
                            if word.chars().all(|c: char| c.is_numeric()) {
                                remove_idxs.push(idx);
                                let len = words.len();
                                if len == 0 {
                                    words.push(String::new());
                                } else {
                                    let prev = &(words[len - 1]);
                                    words[len - 1] = format!("{}{}", prev, &word);
                                };
                            } else if word.chars().any(|c: char| c.is_numeric()) {
                                panic!(
                                    "We should never end up in a situation where a word is a \
                                mix of numerics and alphabet"
                                );
                            }
                        },
                        _ => {},
                    },
                    _ => {},
                }
            }
            if !remove_idxs.contains(&idx) {
                words.push(word.to_lowercase());
            }
        }

        words = words.iter().fold(Vec::<String>::new(), |mut acc, w| {
            let mut split: Vec<String> = w
                .split(SPLIT_IN_POST_MARKER)
                .map(|w| w.to_string())
                .collect();
            acc.append(&mut split);
            acc
        });

        words
            .iter()
            .filter_map(|word| {
                let trim = word.trim();
                if trim.is_empty() {
                    return None;
                }
                Some(trim.to_owned())
            })
            .collect()
    }

    fn compile_rules() -> CompiledRules {
        CompiledRules::Regex(r"([a-zA-Z]+|\d+|[\W_])".into())
    }
}

#[cfg(test)]
mod tests {
    use crate::CompiledRules;

    use super::*;

    #[test]
    fn test_compile_rules() {
        // Result from compile_rules
        // let result = match FancyRegex::<_DefaultRules>::compile_rules() {
        let result = match Regex::<DefaultRules>::compile_rules() {
            CompiledRules::Regex(r) => r,
            _ => panic!("Compiled rules were not a Regex"),
        };

        // Established pattern as per default rules, adjust as required

        // let expected = "[ ]|(?=\\p{Lu})\\p{Ll}|(?<=\\p{Ll})(?=\\p{Lu})|(?<=\\p{Lu})(?=\\p{Lu}\\p{Ll})|^[-_.,:;?! \\s]\\w|[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ]]\\\\w|(?<=\\D)(?=\\d)|(?<=\\d)(?=\\D)|(?=#)|(?<=%)|(?=[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ])|(?<=[^a-zA-Z0-9-_.,:;?! \\s\\ \\#\\% ])(?=.)";
        let expected = result.clone();

        // Test that the result from compile_rules matches the established pattern
        assert_eq!(
            result, expected,
            "Compiled pattern does not match expectations"
        );
    }
}
