//! The rules, reduced once per call to what the walk actually asks of them.
//!
//! A ruleset is a list of [`ResolverProcessingRule`], and answering "does this rule apply to this
//! character" by searching that list means comparing against every entry, for every rule kind, for
//! every character. The list does not change while a string is being walked, so the answers are
//! worked out once here and the walk reads flags instead.

use alloc::vec::Vec;

use crate::rules::RemoveMode::{All, Ends, Middle};
use crate::rules::ResolverProcessingRule::{BoundEnd, BoundStart, Remove};
use crate::rules::Scope::{FullInput, SingleWord};
use crate::rules::{ResolverProcessingRule, RuleTarget};

/// Membership of a character in a set, as a pair of bitmaps over the ASCII range.
///
/// The sets in play are punctuation and the special characters, both of which are ASCII, so
/// membership is a bit test rather than a scan of a string.
#[derive(Default, Clone, Copy)]
pub(crate) struct AsciiSet {
    low: u64,
    high: u64,
}

impl AsciiSet {
    pub(crate) fn from_chars(chars: &str) -> Self {
        let mut set = AsciiSet::default();
        for c in chars.chars() {
            set.insert(c);
        }
        set
    }

    #[inline]
    fn insert(&mut self, c: char) {
        let value = c as u32;
        if value < 64 {
            self.low |= 1u64 << value;
        } else if value < 128 {
            self.high |= 1u64 << (value - 64);
        }
        // outside ASCII nothing is recorded: neither set contains such a character
    }

    #[inline]
    pub(crate) fn contains(&self, c: char) -> bool {
        let value = c as u32;
        if value < 64 {
            self.low & (1u64 << value) != 0
        } else if value < 128 {
            self.high & (1u64 << (value - 64)) != 0
        } else {
            false
        }
    }
}

/// What the ruleset says about one target.
#[derive(Default, Clone, Copy)]
pub(crate) struct TargetRules {
    pub(crate) remove_all: bool,
    pub(crate) remove_middle_input: bool,
    pub(crate) remove_middle_word: bool,
    pub(crate) remove_ends_input: bool,
    pub(crate) remove_ends_word: bool,
    pub(crate) bound_start: bool,
    pub(crate) bound_end: bool,
}

impl TargetRules {
    fn of(rules: &[ResolverProcessingRule], target: &RuleTarget) -> Self {
        let mut this = TargetRules::default();
        for rule in rules {
            match rule {
                Remove(t, mode) if t == target => match mode {
                    All => this.remove_all = true,
                    Middle(FullInput) => this.remove_middle_input = true,
                    Middle(SingleWord) => this.remove_middle_word = true,
                    Ends(FullInput) => this.remove_ends_input = true,
                    Ends(SingleWord) => this.remove_ends_word = true,
                    _ => (),
                },
                BoundStart(t) if t == target => this.bound_start = true,
                BoundEnd(t) if t == target => this.bound_end = true,
                _ => (),
            }
        }
        this
    }

    /// Whether this target is mentioned at all, so the walk can skip testing for it.
    #[inline]
    pub(crate) fn is_inert(&self) -> bool {
        !(self.remove_all
            || self.remove_middle_input
            || self.remove_middle_word
            || self.remove_ends_input
            || self.remove_ends_word
            || self.bound_start
            || self.bound_end)
    }
}

/// The whole ruleset, in the form the walk reads.
pub(crate) struct Compiled {
    pub(crate) punct: AsciiSet,
    pub(crate) non_punct_special: AsciiSet,
    pub(crate) punct_char: TargetRules,
    pub(crate) punct_run: TargetRules,
    pub(crate) numerics: TargetRules,
    pub(crate) non_punct_special_rules: TargetRules,
    pub(crate) case_change: TargetRules,
    /// Rules naming one particular character, which are few and are checked in order.
    pub(crate) chars: Vec<(char, TargetRules)>,
}

impl Compiled {
    pub(crate) fn new(
        rules: &[ResolverProcessingRule],
        punct_chars: &str,
        non_punct_special_chars: &str,
    ) -> Self {
        let mut chars = Vec::new();
        for rule in rules {
            if let Some(RuleTarget::Char(c)) = rule.target() {
                if !chars.iter().any(|(seen, _)| seen == c) {
                    chars.push((*c, TargetRules::of(rules, &RuleTarget::Char(*c))));
                }
            }
        }

        Compiled {
            punct: AsciiSet::from_chars(punct_chars),
            non_punct_special: AsciiSet::from_chars(non_punct_special_chars),
            punct_char: TargetRules::of(rules, &RuleTarget::PunctSpecialChar),
            punct_run: TargetRules::of(rules, &RuleTarget::PunctSpecialCharRun),
            numerics: TargetRules::of(rules, &RuleTarget::Numerics),
            non_punct_special_rules: TargetRules::of(rules, &RuleTarget::NonPunctSpecialChar),
            case_change: TargetRules::of(rules, &RuleTarget::CaseChangeNonAcronym),
            chars,
        }
    }
}
