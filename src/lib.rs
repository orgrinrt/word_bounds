//! Splitting a string into the words a human would say it contains.
//!
//! `camelCase`, `snake_case`, `SCREAMING_CASE`, `kebab-case`, `Title Case`, punctuation
//! runs, digit boundaries and any mixture of them, resolved by a rule set rather than by a
//! fixed pattern, so the rules can be replaced without replacing the walker.
//!
//! Three backends compute the same answer. [`Charwalk`](impls::charwalk::Charwalk) walks
//! the characters and takes no dependency at all; the regex backends compile the rules to a
//! pattern and are faster on long input. Which one is present is a feature.
//!
//! # Allocation
//!
//! Three positions, and each is a feature.
//!
//! | Feature | What is available |
//! |---|---|
//! | default | Everything. The regex backends, the shared compiled pattern, `Vec<String>` output. |
//! | `no_std` | `Charwalk` and `alloc`. The regex backends need `std` and are compiled out. |
//! | `no_alloc` | Adds `fill_words`, which writes into storage the caller lends and allocates nothing. |
//!
//! `no_alloc` implies `no_std` and does not take the allocating API away: a crate that has
//! an allocator and wants the lending form for one hot path enables it and keeps both.

#![cfg_attr(feature = "no_std", no_std)]

// Both regex crates need `std`, so `no_std` cannot have them, and asking for both used to
// build clean and silently fall back to the character walker. A consumer got different words
// out of the same input with no error and nothing to grep for, and because cargo unifies
// features across a dependency graph it did not have to be the consumer who asked for
// `no_std`: any sibling anywhere in the graph enabling it was enough.
//
// A refusal that names the fix is the honest form of a combination that cannot work.
#[cfg(all(feature = "no_std", feature = "use_regex"))]
compile_error!(
    "word_bounds: `use_regex` and `no_std` are exclusive, and both are on. The regex backend \
     needs std. Drop `use_regex` for the character walker, which is what `no_std` leaves, or \
     drop `no_std`. Note that cargo unifies features across a dependency graph, so `no_std` \
     may have been enabled by a sibling rather than by you: `cargo tree -e features` names it."
);

#[cfg(all(feature = "no_std", feature = "use_fancy_regex"))]
compile_error!(
    "word_bounds: `use_fancy_regex` and `no_std` are exclusive, and both are on. The \
     fancy-regex backend needs std. Drop `use_fancy_regex` for the character walker, which is \
     what `no_std` leaves, or drop `no_std`. Note that cargo unifies features across a \
     dependency graph, so `no_std` may have been enabled by a sibling rather than by you: \
     `cargo tree -e features` names it."
);

// `alloc` rather than `std`, in both configurations. It is a sysroot crate, so naming it
// here costs a std build nothing and is what lets one set of paths serve both.
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::rules::{DefaultRules, ResolverRules};

pub mod impls;
pub mod resolver;
pub mod rules;
pub mod sink;

#[cfg(feature = "no_alloc")]
mod lending;
#[cfg(feature = "no_alloc")]
pub use lending::{fill_words, fill_words_with, Segmented};

// The lending API answers in notko's types, so they are re-exported here. Without this a
// consumer has to take a direct dependency on notko in order to name the thing this crate
// handed it, which is a dependency it did not choose and would have to keep in step.
#[cfg(feature = "no_alloc")]
pub use notko::lend::{Exhausted, Fill, Lend};
#[cfg(feature = "no_alloc")]
pub use notko::outcome::Outcome;

#[cfg(feature = "optimize_for_cpu")]
pub(crate) const CHARS_PER_WORD_AVG: usize = 3;

#[cfg(all(not(feature = "optimize_for_cpu"), feature = "optimize_for_memory"))]
pub(crate) const CHARS_PER_WORD_AVG: u8 = 3;

/// One way of finding the word bounds in a string.
///
/// The rule set `R` says what a bound is; the implementor says how to look for one. Both
/// answers agree, which is what makes the backends interchangeable and what the
/// cross-backend tests assert.
pub trait WordBoundResolverImpl<R: ResolverRules = DefaultRules> {
    /// The words in `s`, lowercased, with the delimiters between them dropped.
    fn resolver(s: &str) -> Vec<String>;

    /// The rule set compiled to whatever form this backend consumes.
    fn compile_rules() -> CompiledRules;
}

/// A rule set in the form its backend wants it.
///
/// [`NotApplicable`](CompiledRules::NotApplicable) is what a backend returns when it reads
/// the rules directly and has nothing to compile, which is the character walker's answer.
pub enum CompiledRules {
    /// A regular expression, for a backend that runs one.
    Regex(String),
    /// A plain string of characters, for a backend that matches against a set.
    Str(String),
    /// Nothing to compile.
    NotApplicable,
}
