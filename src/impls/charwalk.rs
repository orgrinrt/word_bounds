use core::marker::PhantomData;

use alloc::string::String;
use alloc::vec::Vec;

use crate::impls::compiled::Compiled;
use crate::rules::{DefaultRules, ResolverRules};
use crate::CompiledRules::NotApplicable;
use crate::sink::WordSink;
use crate::{CompiledRules, WordBoundResolverImpl, __str_ext__instance_words_vec};

macro_rules! __str_ext__impl_parsing_for_target {
    ($target:expr, $predicate:expr, $rules:ident, $del_flag:ident, $commit_flag:ident,
    $bstart:ident, $bend:ident,
    $is_first:expr, $is_last:expr) => {
        __str_ext__impl_parsing_for_target!($target, $predicate, $rules, $del_flag, $commit_flag,
        $bstart, $bend, $is_first, $is_last, { {} });
    };
    ($target:expr, $predicate:expr, $rules:ident, $del_flag:ident, $commit_flag:ident,
    $bstart:ident, $bend:ident,
    $is_first:expr, $is_last:expr, { $($extras:tt)* } ) => {
        if !$target.is_inert() && $predicate && !$del_flag {
            if $target.remove_all {
                $del_flag = true;
            } else if $target.remove_middle_input {
                if !$is_first && !$is_last {
                    $del_flag = true;
                }
            } else if $target.remove_middle_word {
                unimplemented!();
            } else if $target.remove_ends_input {
                if $is_first || $is_last {
                    $del_flag = true;
                }
            } else if $target.remove_ends_word {
                unimplemented!();
            }
            if $target.bound_start {
                $bstart = true;
                $commit_flag = true;
            }
            if $target.bound_end {
                $bend = true;
                $commit_flag = true;
            }
            $($extras)*
        }
    };
}

/// Whether `c` is a digit, without reaching for the Unicode tables when it is ASCII.
#[inline]
fn is_digit(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_digit()
    } else {
        c.is_numeric()
    }
}

/// Whether `c` is upper case, ASCII answered without the tables.
#[inline]
fn is_upper(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_uppercase()
    } else {
        c.is_uppercase()
    }
}

/// Whether `c` is lower case, ASCII answered without the tables.
#[inline]
fn is_lower(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_lowercase()
    } else {
        c.is_lowercase()
    }
}

pub struct Charwalk<R: ResolverRules = DefaultRules> {
    _phantom_data: PhantomData<R>,
}

impl<R: ResolverRules> WordBoundResolverImpl<R> for Charwalk<R> {
    fn resolver(s: &str) -> Vec<String> {
        __str_ext__instance_words_vec!(s, words);
        let mut sink = VecSink { pending: String::new(), words };
        // Infallible: a `Vec` sink refuses nothing.
        let _ = walk::<R, _>(s, &mut sink);
        sink.words
    }

    fn compile_rules() -> CompiledRules {
        NotApplicable
    }
}

/// Walks `s` and hands every word it finds to `sink`.
///
/// The one implementation of the segmentation. `resolver` calls it with a sink that
/// collects into a `Vec<String>`; under `no_alloc` the lending API calls it with one that
/// writes into storage the caller supplied. Stops at the first refusal from the sink, which
/// is how a lend that ran out is reported rather than truncated over.
pub(crate) fn walk<R: ResolverRules, S: WordSink>(s: &str, sink: &mut S) -> Result<(), S::Err> {
        let punct_chars = R::punct_chars_non_regex();
        let non_punct_special_chars = R::non_punct_special_chars_non_regex();
        let rule_list = R::resolution_pass_rules();
        let rules = Compiled::new(&rule_list, &punct_chars, &non_punct_special_chars);

        let mut prev_char: Option<char> = None;
        // Kept and maintained, read by nothing yet. Two rule targets in this walker are
        // `unimplemented!()`, `remove_middle_word` and `remove_ends_word`, and both are
        // about where a character sits inside the word being built rather than inside the
        // input. That is the question these two answer, so they are the state those rules
        // will need and are tracked now rather than reconstructed later. Underscored so the
        // build stays quiet about a gap that is deliberate.
        let mut _prev_prev_char: Option<char> = None;
        let mut _prev_committed_char: Option<char> = None;
        let mut _prev_was_split: i8 = 0;
        // Same: the counter is maintained, and the predicate over it has no caller yet.
        // Not underscored, which would shadow the counter it reads and turn every later
        // arithmetic use of that name into a closure.
        #[allow(unused_variables)]
        let prev_was_split = || _prev_was_split == 1;

        let mut walk = s.chars().peekable();
        let mut idx = 0usize;
        while let Some(c) = walk.next() {
            // Loop-local, every one of them. Each is set here before anything reads it, so
            // declaring them outside only ever wrote an initial value nobody looked at.
            let mut flag_to_commit = false;
            let mut flag_to_delete = false;
            let mut bound_start = false;
            let mut bound_end = false;
            let next_char = walk.peek().copied();
            let is_first = idx == 0;
            let is_last = next_char.is_none();

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
                        is_first,
                        is_last
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
                        is_first,
                        is_last,
                        $extras
                    );
                };
            }

            let same_before = prev_char == Some(c);
            let same_after = next_char == Some(c);
            let punct_run = rules.punct.contains(c) && (same_before || same_after);
            let (in_run, run_starts, run_ends) = (
                punct_run,
                punct_run && !same_before,
                punct_run && !same_after,
            );
            impl_parsing_for!(rules.punct_run, in_run, {
                {
                    // the run bounds the token it forms, not each character inside it
                    if !run_starts {
                        bound_start = false;
                    }
                    if !run_ends {
                        bound_end = false;
                    }
                    if !run_starts && !run_ends {
                        flag_to_commit = false;
                    }
                }
            });
            impl_parsing_for!(rules.punct_char, !in_run && rules.punct.contains(c));
            impl_parsing_for!(rules.numerics, is_digit(c), {
                {
                    if (prev_char.is_some() && is_digit(prev_char.unwrap()))
                        && (next_char.is_some() && is_digit(next_char.unwrap()))
                    {
                        flag_to_commit = false;
                        bound_start = false;
                        bound_end = false;
                    } else if (prev_char.is_some() && !is_digit(prev_char.unwrap()))
                        && ((next_char.is_some() && is_digit(next_char.unwrap()))
                            || next_char.is_none())
                    {
                        bound_end = false;
                        if rules.numerics.bound_start {
                            bound_start = true;
                        }
                    } else if (prev_char.is_some() && is_digit(prev_char.unwrap()))
                        && ((next_char.is_some() && !is_digit(next_char.unwrap()))
                            || next_char.is_none())
                    {
                        bound_start = false;
                        if rules.numerics.bound_end {
                            bound_end = true;
                            flag_to_commit = false;
                        }
                    }
                }
            });
            for (inner_c, char_rules) in &rules.chars {
                impl_parsing_for!(*char_rules, c == *inner_c, { {} });
            }
            impl_parsing_for!(rules.non_punct_special_rules, rules.non_punct_special.contains(c));
            impl_parsing_for!(
                rules.case_change,
                prev_char.is_some()
                    && ((is_upper(prev_char.unwrap())
                        && (next_char.is_some() && is_lower(next_char.unwrap()))
                        && is_upper(c))
                        || (is_lower(prev_char.unwrap()) && is_upper(c)))
            );

            // process

            if !flag_to_delete && (!flag_to_commit || (flag_to_commit && bound_end)) {
                _prev_committed_char = Some(c);
                if !bound_start {
                    sink.push_char(c)?;
                }
            }
            if flag_to_commit {
                if !sink.pending_is_empty() {
                    sink.commit()?;
                }
                _prev_was_split = 2;
            }
            if !flag_to_delete && flag_to_commit && bound_start {
                _prev_committed_char = Some(c);
                sink.push_char(c)?;
            }
            if is_last || (!flag_to_delete && bound_start && bound_end) {
                // a character that ended a token without starting one has already been committed
                // above, so there is nothing left pending: the end of a punctuation run is the
                // case that reaches here with an empty word in hand.
                let already_committed =
                    flag_to_commit && bound_end && !bound_start && !flag_to_delete;
                if !already_committed {
                    if !flag_to_delete && !sink.pending_ends_with(c) {
                        sink.push_char(c)?;
                    }
                    if !sink.pending_is_empty() {
                        sink.commit()?;
                    }
                }
            }
            _prev_prev_char = prev_char;
            prev_char = Some(c);
            idx += 1;
            _prev_was_split = if _prev_was_split < 1 { 0 } else { _prev_was_split - 1 };
        }

        Ok(())
}

/// The sink behind [`Charwalk::resolver`]: a `String` being built, and the words so far.
struct VecSink {
    pending: String,
    words:   Vec<String>,
}

impl WordSink for VecSink {
    type Err = core::convert::Infallible;

    fn push_char(&mut self, c: char) -> Result<(), Self::Err> {
        self.pending.push(c);
        Ok(())
    }

    fn pending_is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    fn pending_ends_with(&self, c: char) -> bool {
        self.pending.ends_with(c)
    }

    fn commit(&mut self) -> Result<(), Self::Err> {
        // `str::to_lowercase` rather than mapping each character, because it is the one
        // that applies the final-sigma rule. The lending sink cannot use it and reproduces
        // the rule by hand; `tests/lowercase_parity.rs` asserts the two agree.
        self.words.push(self.pending.to_lowercase());
        self.pending.clear();
        Ok(())
    }
}
