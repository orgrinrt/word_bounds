//! Segmenting into storage the caller lends, without allocating.
//!
//! The allocating API answers with a `Vec<String>`, which is an allocation per word and one
//! for the vector. This one asks for two lends instead: somewhere to put the characters and
//! somewhere to put the bounds between them. Neither has to come from an allocator, so a
//! stack array, a slice out of an arena, or a region from an allocator the caller already
//! holds all work, and this never asks which it was given.

use notko::lend::{Exhausted, Fill, Lend};
use notko::outcome::Outcome;

use crate::rules::{DefaultRules, ResolverRules};
use crate::sink::{is_cased, WordSink, FINAL_SIGMA, NON_FINAL_SIGMA};

/// The words a segmentation found, in the storage it was lent.
///
/// Borrows both lends for as long as it lives, which is what lets a word come back as a
/// `&str` without copying: a word is a slice of the text lend, delimited by a pair in the
/// bounds lend.
#[derive(Debug, Clone, Copy)]
pub struct Segmented<'a> {
    text:   &'a [u8],
    bounds: &'a [(usize, usize)],
}

impl<'a> Segmented<'a> {
    /// How many words were found.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bounds.len()
    }

    /// Whether no words were found, which is what an empty input gives.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bounds.is_empty()
    }

    /// The word at `index`, or `None` past the end.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&'a str> {
        let &(start, end) = self.bounds.get(index)?;
        // The walker writes whole characters and a bound falls between two of them, so
        // every slice here is on a character boundary and is valid UTF-8 by construction.
        // Checked anyway: it is a scan of memory this crate wrote a moment ago, and being
        // wrong about that would be unsound rather than merely incorrect.
        core::str::from_utf8(self.text.get(start .. end)?).ok()
    }

    /// Every word, in the order they appeared.
    pub fn iter(&self) -> impl Iterator<Item = &'a str> + '_ {
        (0 .. self.len()).filter_map(|i| self.get(i))
    }

    /// The characters of every word, run together with nothing between them.
    ///
    /// Useful for measuring how much of the text lend was actually needed.
    #[must_use]
    pub const fn text_len(&self) -> usize {
        self.text.len()
    }
}

/// Splits `s` into words, writing them into storage the caller supplies.
///
/// `text` receives the characters of every word, lowercased and run together with no
/// separator. `bounds` receives one `(start, end)` pair per word, indexing into what landed
/// in `text`. Together they are a [`Segmented`], which hands back each word as a `&str`.
///
/// Refuses rather than truncating, on either lend. An [`Exhausted`] says how much was
/// wanted against how much was there, so a caller doubling a buffer converges instead of
/// guessing. What it reports is a lower bound: the shortfall is found partway through, so
/// `wanted` is what had been reached rather than the full requirement.
///
/// `?Sized` on both, so a bare `&mut [u8]` out of an arena is lent directly rather than by
/// lending a reference to one.
///
/// ```
/// use word_bounds::fill_words;
///
/// let mut text = [0u8; 32];
/// let mut bounds = [(0, 0); 8];
///
/// let words = fill_words("someHTTPRequest_id", &mut text, &mut bounds).unwrap();
///
/// assert_eq!(words.len(), 4);
/// assert_eq!(words.get(0), Some("some"));
/// assert_eq!(words.get(3), Some("id"));
/// ```
///
/// A lend too small to hold the answer says so rather than handing back a short one:
///
/// ```
/// use word_bounds::fill_words;
///
/// let mut text = [0u8; 32];
/// let mut bounds = [(0, 0); 2];
///
/// let refused = fill_words("someHTTPRequest_id", &mut text, &mut bounds).unwrap_err();
///
/// assert_eq!(refused.had, 2);
/// assert!(refused.wanted > refused.had);
/// ```
pub fn fill_words<'a, T, B>(
    s: &str,
    text: &'a mut T,
    bounds: &'a mut B,
) -> Outcome<Segmented<'a>, Exhausted>
where
    T: Lend<u8> + ?Sized,
    B: Lend<(usize, usize)> + ?Sized,
{
    fill_words_with::<DefaultRules, T, B>(s, text, bounds)
}

/// [`fill_words`], against a rule set of your own.
///
/// The rules say what a bound is. [`DefaultRules`] is what `fill_words` uses and what the
/// allocating API uses, so the two agree; another rule set changes both together.
pub fn fill_words_with<'a, R, T, B>(
    s: &str,
    text: &'a mut T,
    bounds: &'a mut B,
) -> Outcome<Segmented<'a>, Exhausted>
where
    R: ResolverRules,
    T: Lend<u8> + ?Sized,
    B: Lend<(usize, usize)> + ?Sized,
{
    let mut sink = LendingSink {
        text:          Fill::new(text),
        bounds:        Fill::new(bounds),
        pending_start: 0,
        held:          None,
        before_held:   None,
    };

    if let Err(exhausted) = crate::impls::charwalk::walk::<R, _>(s, &mut sink) {
        return Outcome::Err(exhausted);
    }

    Outcome::Ok(Segmented { text: sink.text.finish(), bounds: sink.bounds.finish() })
}

/// A sink writing into two lends: the characters, and the bounds between words.
///
/// Holds the most recent character back rather than writing it immediately, because
/// lowercasing a capital sigma depends on whether it turns out to be the last character of
/// its word, and that is not known until the next one arrives or the word ends. Holding one
/// character costs nothing and means no byte written here is ever revised.
struct LendingSink<'a> {
    text:          Fill<'a, u8>,
    bounds:        Fill<'a, (usize, usize)>,
    /// Where in `text` the word being built starts.
    pending_start: usize,
    /// The character pushed most recently, not yet written.
    held:          Option<char>,
    /// The one before it, which the final-sigma rule needs.
    before_held:   Option<char>,
}

impl LendingSink<'_> {
    /// Writes one character's UTF-8 into the text lend.
    fn write(&mut self, c: char) -> Result<(), Exhausted> {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        match self.text.extend(encoded.as_bytes().iter().copied()) {
            Outcome::Ok(()) => Ok(()),
            Outcome::Err(e) => Err(e),
        }
    }

    /// Writes the held character, lowercased, treating it as final or not as told.
    fn flush_held(&mut self, is_final: bool) -> Result<(), Exhausted> {
        let Some(c) = self.held.take() else {
            return Ok(());
        };

        // The one place `char::to_lowercase` and `str::to_lowercase` disagree: a capital
        // sigma at the end of a word, with a cased letter before it, takes the final form.
        // Both forms are named here rather than one being left to `to_lowercase`, so the
        // pair is visible in one place; `sink::tests` pins that they are what this assumes.
        if c == 'Σ' {
            let final_position = is_final && self.before_held.is_some_and(is_cased);
            return self.write(if final_position { FINAL_SIGMA } else { NON_FINAL_SIGMA });
        }

        for lowered in c.to_lowercase() {
            self.write(lowered)?;
        }
        Ok(())
    }
}

impl WordSink for LendingSink<'_> {
    type Err = Exhausted;

    fn push_char(&mut self, c: char) -> Result<(), Self::Err> {
        // Read before the flush, which takes it. Reading after gives `None` every time,
        // and the final-sigma rule then never fires because it never sees a letter before
        // the sigma.
        let was_held = self.held;
        // Whatever was held is no longer last, so it is written in its non-final form.
        self.flush_held(false)?;
        self.before_held = was_held;
        self.held = Some(c);
        Ok(())
    }

    fn pending_is_empty(&self) -> bool {
        self.text.len() == self.pending_start && self.held.is_none()
    }

    fn pending_ends_with(&self, c: char) -> bool {
        self.held == Some(c)
    }

    fn commit(&mut self) -> Result<(), Self::Err> {
        self.flush_held(true)?;
        let end = self.text.len();

        match self.bounds.push((self.pending_start, end)) {
            Outcome::Ok(()) => {},
            Outcome::Err(e) => return Err(e),
        }
        self.pending_start = end;
        self.before_held = None;
        Ok(())
    }
}
