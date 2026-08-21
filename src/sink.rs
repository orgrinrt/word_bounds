//! Where a walk puts the words it finds.
//!
//! The walk is the hard part and there is one of it. What differs between the allocating
//! API and the lending one is only where a committed word goes, so that is what this
//! separates out: the walker pushes characters and says when a word ended, and the sink
//! decides whether that means a `String` in a `Vec` or bytes in storage somebody lent.
//!
//! It is the extension point as well. A consumer that wants neither shape writes its own
//! [`WordSink`] and hands it to [`walk`](crate::impls::charwalk::walk), which is how a case
//! converter emits its separators and capitals as the words arrive rather than segmenting
//! first and rewriting afterwards.
//!
//! A sink that lowercases character by character, as one writing into fixed storage must,
//! needs the final-sigma rule reproduced. [`FINAL_SIGMA`], [`NON_FINAL_SIGMA`] and
//! [`is_cased`] are what that takes, and are public for exactly that reason.

use core::fmt::Debug;

/// Somewhere a walk can put the words it finds.
///
/// A word is built up by [`push_char`](WordSink::push_char) and ended by
/// [`commit`](WordSink::commit).
///
/// # What an implementor owes
///
/// The two predicates are answers the walk acts on rather than conveniences, so an
/// implementor that answers them wrongly changes the output. `pending_is_empty` is what the
/// walk asks before committing, because two of its paths reach a commit with nothing in
/// hand, and an empty word is not a word: a sink whose answer is always `false` yields an
/// empty leading word on any input that starts with a delimiter, where the crate's own
/// sinks yield none.
///
/// `pending_ends_with` guards one push at the end of the input, so that a character which
/// both ends a token and starts one is not written twice.
///
/// # What this crate's sinks do, and what a different one may
///
/// Both sinks here lowercase on commit, which is what the crate promises about its output.
/// That is a property of those sinks rather than of this trait: a sink written for a case
/// conversion capitalises instead, which is exactly what
/// [`str_extensions`](https://github.com/orgrinrt/str_extensions)' does, and it is a
/// legitimate implementation. The trait says when a word starts and ends; what goes into
/// the output is the sink's business.
pub trait WordSink {
    /// What this sink refuses with. `Infallible` for a sink that cannot fail.
    type Err: Debug;

    /// Adds one character to the word being built.
    fn push_char(&mut self, c: char) -> Result<(), Self::Err>;

    /// Whether nothing has been pushed since the last commit.
    ///
    /// The walk asks before committing, so answering wrongly produces or swallows words.
    fn pending_is_empty(&self) -> bool;

    /// Whether the word being built ends with `c`.
    fn pending_ends_with(&self, c: char) -> bool;

    /// Ends the current word and starts an empty one.
    ///
    /// The walk calls this only when [`pending_is_empty`](WordSink::pending_is_empty)
    /// answered `false`, which makes that answer part of the contract rather than a
    /// courtesy.
    fn commit(&mut self) -> Result<(), Self::Err>;
}

/// Lowercases one character the way `str::to_lowercase` would, except for final sigma.
///
/// Rust's `str::to_lowercase` differs from mapping each character on its own in exactly one
/// place: a capital sigma at the end of a word lowercases to the final form `ς` rather than
/// to `σ`. A sink that lowercases as it goes does not know a character is final at the
/// moment it arrives, so it holds the most recent one back and writes it once the next one
/// arrives or the word ends, at which point which form to use is settled.
pub const NON_FINAL_SIGMA: char = 'σ';
/// The form a capital sigma takes at the end of a word.
pub const FINAL_SIGMA: char = 'ς';

/// Whether a character is cased, in the sense the final-sigma rule means.
///
/// Unicode's `Cased` property is `Lowercase` or `Uppercase` or general category `Lt`, plus
/// the `Other_*` additions. `char::is_lowercase` and `char::is_uppercase` cover the first
/// two. They do not cover titlecase: `ǅ` is `Lt` and answers false to both, so a sigma
/// after it was treated as though nothing cased preceded it.
///
/// A character with a case mapping that changes it is cased, which is what the last two
/// arms test and what brings `Lt` in. Between them the four cover `Cased` for every
/// character that has a case mapping at all.
///
/// The remainder is the handful of `Cased` characters with no mapping of their own, which
/// no rule here can reach without a Unicode table this crate does not carry.
/// `tests/lending_parity.rs` pins the cases that motivated each arm.
pub fn is_cased(c: char) -> bool {
    c.is_lowercase()
        || c.is_uppercase()
        || c.to_lowercase().next() != Some(c)
        || c.to_uppercase().next() != Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_sigmas_are_what_the_rule_needs_them_to_be() {
        // Mapping the character on its own gives the non-final form, which is why a sink
        // that lowercases as it goes has to put the final one back at a word end.
        let mapped: alloc::vec::Vec<char> = 'Σ'.to_lowercase().collect();
        assert_eq!(mapped, [NON_FINAL_SIGMA]);

        // Both are lower case sigma, and they differ.
        assert_ne!(NON_FINAL_SIGMA, FINAL_SIGMA);
        assert_eq!(NON_FINAL_SIGMA.to_uppercase().next(), Some('Σ'));
        assert_eq!(FINAL_SIGMA.to_uppercase().next(), Some('Σ'));

        // Same width in UTF-8. An earlier version of the lending sink overwrote the
        // non-final form in place, which is only sound because of this.
        assert_eq!(NON_FINAL_SIGMA.len_utf8(), FINAL_SIGMA.len_utf8());
    }

    #[test]
    fn is_cased_answers_for_the_letters_the_rule_asks_about() {
        assert!(is_cased('a'));
        assert!(is_cased('Z'));
        assert!(is_cased('Ο'));
        assert!(is_cased('ς'));

        // The rule needs a cased letter before the sigma, so these are the cases where a
        // trailing sigma stays in the ordinary form.
        assert!(!is_cased('1'));
        assert!(!is_cased('_'));
        assert!(!is_cased(' '));
        assert!(!is_cased('日'));
    }
}
