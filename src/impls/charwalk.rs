use std::marker::PhantomData;

use crate::rules::RemoveMode::{All, Ends, Middle};
use crate::rules::ResolverProcessingRule::{BoundEnd, BoundStart, Remove};
use crate::rules::RuleTarget::{
    CaseChangeNonAcronym, Char, NonPunctSpecialChar, Numerics, PunctSpecialChar,
};
use crate::rules::Scope::{FullInput, SingleWord};
use crate::rules::{DefaultRules, ResolverRules};
use crate::CompiledRules::NotApplicable;
use crate::{CompiledRules, WordBoundResolverImpl, __str_ext__instance_words_vec};

macro_rules! __str_ext__impl_parsing_for_target {
    ($target:expr, $predicate:expr, $rules:ident, $del_flag:ident, $commit_flag:ident,
    $bstart:ident, $bend:ident,
    $idx:ident, $last_idx:ident) => {
        __str_ext__impl_parsing_for_target!($target, $predicate, $rules, $del_flag, $commit_flag,
        $bstart, $bend, $idx, $last_idx, { {} });
    };
    ($target:expr, $predicate:expr, $rules:ident, $del_flag:ident, $commit_flag:ident,
    $bstart:ident, $bend:ident,
    $idx:ident, $last_idx:ident, { $($extras:tt)* } ) => {
        if $predicate && !$del_flag {
            if $rules.contains(&Remove($target, All)) {
                $del_flag = true;
            } else if $rules.contains(&Remove($target, Middle(FullInput))) {
                if $idx != 0 && $idx != $last_idx {
                    $del_flag = true;
                }
            } else if $rules.contains(&Remove($target, Middle(SingleWord))) {
                unimplemented!();
            } else if $rules.contains(&Remove($target, Ends(FullInput))) {
                if $idx == 0 || $idx == $last_idx {
                    $del_flag = true;
                }
            } else if $rules.contains(&Remove($target, Ends(SingleWord))) {
                unimplemented!();
            }
            if $rules.contains(&BoundStart($target)) {
                $bstart = true;
                $commit_flag = true;
            }
            if $rules.contains(&BoundEnd($target)) {
                $bend = true;
                $commit_flag = true;
            }
            $($extras)*
        }
    };
}

pub struct Charwalk<R: ResolverRules = DefaultRules> {
    _phantom_data: PhantomData<R>,
}

impl<R: ResolverRules> WordBoundResolverImpl<R> for Charwalk<R> {
    fn resolver(s: &str) -> Vec<String> {
        __str_ext__instance_words_vec!(s, words);
        // dbg!(s);

        let punct_chars = R::punct_chars_non_regex();
        let non_punct_special_chars = /*dbg!(*/R::non_punct_special_chars_non_regex()/*)*/;
        let rules = R::resolution_pass_rules();

        let mut prev_char: Option<char> = None;
        let mut prev_prev_char: Option<char> = None;
        let mut prev_committed_char: Option<char> = None;
        let mut next_char: Option<char> = None;
        let chars: Vec<char> = s.chars().collect();
        let chars_len = chars.len();
        let last_idx = chars_len - 1;
        let mut flag_to_commit = false;
        let mut curr_word = String::new();
        let mut _prev_was_split: i8 = 0;
        let prev_was_split = || _prev_was_split == 1;
        let mut flag_to_delete = false;
        let mut bound_start: bool = false;
        let mut bound_end: bool = false;

        for (idx, c) in s.chars().enumerate() {
            flag_to_commit = false;
            flag_to_delete = false;
            bound_start = false;
            bound_end = false;
            next_char = if idx < last_idx { Some(chars[idx + 1]) } else { None };

            macro_rules! impl_parsing_for {
                ($target:expr, $predicate:expr) => {
                    __str_ext__impl_parsing_for_target!(
                        $target,
                        $predicate,
                        rules,
                        flag_to_delete,
                        flag_to_commit,
                        bound_start,
                        bound_end,
                        idx,
                        last_idx
                    );
                };
                ($target:expr, $predicate:expr, { $extras:tt }) => {
                    __str_ext__impl_parsing_for_target!(
                        $target,
                        $predicate,
                        rules,
                        flag_to_delete,
                        flag_to_commit,
                        bound_start,
                        bound_end,
                        idx,
                        last_idx,
                        $extras
                    );
                };
            }

            impl_parsing_for!(PunctSpecialChar, punct_chars.contains(c));
            impl_parsing_for!(Numerics, c.is_numeric(), {
                {
                    if (prev_char.is_some() && prev_char.unwrap().is_numeric())
                        && (next_char.is_some() && next_char.unwrap().is_numeric())
                    {
                        flag_to_commit = false;
                        bound_start = false;
                        bound_end = false;
                    } else if (prev_char.is_some() && !prev_char.unwrap().is_numeric())
                        && ((next_char.is_some() && next_char.unwrap().is_numeric())
                            || next_char.is_none())
                    {
                        bound_end = false;
                        if rules.contains(&BoundStart(Numerics)) {
                            bound_start = true;
                        }
                    } else if (prev_char.is_some() && prev_char.unwrap().is_numeric())
                        && ((next_char.is_some() && !next_char.unwrap().is_numeric())
                            || next_char.is_none())
                    {
                        bound_start = false;
                        if rules.contains(&BoundEnd(Numerics)) {
                            bound_end = true;
                            flag_to_commit = false;
                        }
                    }
                }
            });
            for rule in &rules {
                if let Char(inner_c) = rule.target().unwrap() {
                    impl_parsing_for!(Char(*inner_c), c == *inner_c, { {} });
                }
            }
            impl_parsing_for!(NonPunctSpecialChar, non_punct_special_chars.contains(c));
            impl_parsing_for!(
                CaseChangeNonAcronym,
                prev_char.is_some()
                    && ((prev_char.unwrap().is_uppercase()
                        && (next_char.is_some() && next_char.unwrap().is_lowercase())
                        && c.is_uppercase())
                        || (prev_char.unwrap().is_lowercase() && c.is_uppercase()))
            );

            // process

            if !flag_to_delete && (!flag_to_commit || (flag_to_commit && bound_end)) {
                prev_committed_char = Some(c);
                if !bound_start {
                    curr_word.push(c);
                }
            }
            if flag_to_commit {
                if !curr_word.is_empty() {
                    words.push(curr_word.to_lowercase());
                    curr_word.clear();
                }
                _prev_was_split = 2;
            }
            if !flag_to_delete && flag_to_commit && bound_start {
                prev_committed_char = Some(c);
                curr_word.push(c);
            }
            if idx == last_idx || (!flag_to_delete && bound_start && bound_end) {
                if !curr_word.ends_with(c) {
                    curr_word.push(c);
                }
                words.push(curr_word.to_lowercase());
                curr_word.clear();
            }
            prev_prev_char = prev_char;
            prev_char = Some(c);
            _prev_was_split = if _prev_was_split < 1 { 0 } else { _prev_was_split - 1 };
        }

        words
    }

    fn compile_rules() -> CompiledRules {
        NotApplicable
    }
}
