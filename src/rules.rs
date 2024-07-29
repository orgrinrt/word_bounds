use crate::rules::RemoveMode::{All, Middle};
use crate::rules::ResolverProcessingRule::{BoundEnd, BoundStart, Remove};
use crate::rules::RuleTarget::{
    Acronym, CaseChangeNonAcronym, Char, NonPunctSpecialChar, Numerics, PunctSpecialChar,
};
use crate::rules::Scope::FullInput;

pub trait ResolverRules {
    /// The rules consider the chars in this string punctuation characters, delimiting words.
    ///
    /// This also means inversely, that non-punct chars are everything but these.
    fn punct_chars() -> String {
        String::from(r"\-_\.,:;\?! \ \s")
    }
    fn punct_chars_non_regex() -> String {
        String::from("-_.,:;?! ")
    }
    fn punct_chars_allow_whitespace() -> String {
        String::from(r"\-_\.,:;\?!")
    }

    fn non_punct_special_chars() -> String {
        let mut exclude_chars = Self::punct_chars();

        for rule in &Self::resolution_pass_rules() {
            match rule {
                Remove(Char(c), _) | BoundStart(Char(c)) | BoundEnd(Char(c)) => {
                    exclude_chars.push_str(&format!("{}", *c));
                },
                _ => (),
            }
        }

        format!(r"[^a-zA-Z0-9{}]", exclude_chars)
    }

    fn non_punct_special_chars_non_regex() -> String {
        let mut result: String = String::new();
        let mut exclude_set = Self::punct_chars_non_regex()
            .chars()
            .collect::<std::collections::HashSet<_>>();
        for rule in &Self::resolution_pass_rules() {
            match rule {
                Remove(Char(c), _) | BoundStart(Char(c)) | BoundEnd(Char(c)) => {
                    exclude_set.insert(*c);
                },
                _ => (),
            }
        }
        for i in vec![33..47, 58..64, 91..96, 123..127].into_iter().flatten() {
            if let Some(c) = std::char::from_u32(i as u32) {
                if !exclude_set.contains(&c) {
                    result.push(c);
                }
            }
        }
        result
    }

    fn non_punct_special_chars_allow_whitespace() -> String {
        let mut exclude_chars = Self::punct_chars_allow_whitespace();

        for rule in &Self::resolution_pass_rules() {
            match rule {
                Remove(Char(c), _) | BoundStart(Char(c)) | BoundEnd(Char(c)) => {
                    exclude_chars.push_str(&format!("{}", *c));
                },
                _ => (),
            }
        }

        format!(r"[^a-zA-Z0-9{}]", exclude_chars)
    }
    /// These are operations and rules that are run on the entire input before we pass it on to the
    /// resolution step
    ///
    /// When pre_pass_rules is empty, no pre-process pass will be run.
    fn pre_pass_rules() -> Vec<ResolverProcessingRule> {
        Vec::new()
    }
    /// This is the ruleset passed on to the word bound resolving implementations
    ///
    /// When resolution_pass_rules is empty, the defaults will be used.
    fn resolution_pass_rules() -> Vec<ResolverProcessingRule>;
    /// These are operations and rules that are run on the resolved output of the resolution pass,
    /// which should be a fairly finished vector of properly bounded words.
    ///
    /// When post_pass_rules is empty, no post-process pass will be run.
    fn post_pass_rules() -> Vec<ResolverProcessingRule> {
        Vec::new()
    }
}

pub struct DefaultRules;

impl ResolverRules for DefaultRules {
    fn pre_pass_rules() -> Vec<ResolverProcessingRule> {
        vec![]
    }

    fn resolution_pass_rules() -> Vec<ResolverProcessingRule> {
        vec![
            Remove(PunctSpecialChar, Middle(FullInput)),
            Remove(Char(' '), All),
            BoundStart(CaseChangeNonAcronym),
            BoundEnd(Acronym),
            BoundStart(PunctSpecialChar),
            BoundEnd(PunctSpecialChar),
            BoundStart(Numerics),
            BoundEnd(Numerics),
            // commonly prefixes special chars that are not puncts
            BoundStart(Char('#')),
            // commonly postfixes special chars that are not puncts
            BoundEnd(Char('%')),
            BoundStart(NonPunctSpecialChar),
            BoundEnd(NonPunctSpecialChar),
        ]
    }

    fn post_pass_rules() -> Vec<ResolverProcessingRule> {
        vec![]
    }
}

#[derive(PartialEq)]
pub enum Scope {
    SingleWord,
    FullInput, // i.e prepends only considered for first word, and appends for the last
}

#[derive(PartialEq)]
pub enum RemoveMode {
    None,
    Prepended(Scope),
    Appended(Scope),
    Ends(Scope),
    Middle(Scope),
    All,
}

// pub enum IncludeMode {
//     Dedicated, // as its own word, separately
//     Attach(Direction),
// }

#[derive(PartialEq)]
pub enum RuleTarget {
    Word,
    String(String),
    Char(char),
    Numerics,
    Acronym,
    PunctSpecialChar,
    NonPunctSpecialChar,
    CaseChangeNonAcronym,
}

#[derive(PartialEq)]
pub enum ResolverProcessingRule {
    Remove(RuleTarget, RemoveMode),
    BoundStart(RuleTarget),
    BoundEnd(RuleTarget),
}

impl ResolverProcessingRule {
    pub fn target(&self) -> Option<&RuleTarget> {
        if let Remove(target, _) = self {
            return Some(target);
        } else if let BoundStart(target) = self {
            return Some(target);
        } else if let BoundEnd(target) = self {
            return Some(target);
        }
        return None;
    }
}

#[derive(PartialEq)]
pub enum Direction {
    Previous,
    Next,
    /// e.g hashtags go to next usually (i.e prefix), but
    /// percent symbol would go to previous (i.e postfix)
    ///
    /// For numerics, we'd consider if the next word starts with a case change,
    /// and if it does, it's probably meant to go to next, but if there's
    /// a case change prior, or no case, probably goes to previous
    Auto,
}
