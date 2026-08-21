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
/// [`commit`](WordSink::commit). Committing lowercases what was pushed, which is what the
/// crate promises about its output and belongs here rather than in the walker, since the
/// lending sink cannot lowercase a whole word after the fact without somewhere to put the
/// result.
pub trait WordSink {
    /// What this sink refuses with. `Infallible` for a sink that cannot fail.
    type Err: Debug;

    /// Adds one character to the word being built.
    fn push_char(&mut self, c: char) -> Result<(), Self::Err>;

    /// Whether nothing has been pushed since the last commit.
    fn pending_is_empty(&self) -> bool;

    /// Whether the word being built ends with `c`.
    fn pending_ends_with(&self, c: char) -> bool;

    /// Ends the current word, lowercased, and starts an empty one.
    ///
    /// Called only when there is something pending: the walker checks first, because two
    /// of its paths can reach a commit with an empty word in hand and an empty word is not
    /// a word.
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

/// Whether a character participates in the final-sigma rule as a preceding letter.
///
/// The rule asks for a cased letter before the sigma. Without one, a lone `Σ` is not final
/// and stays `σ`, which is what `str::to_lowercase` does.
pub fn is_cased(c: char) -> bool {
    c.is_lowercase() || c.is_uppercase()
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
