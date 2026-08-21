//! Where the three backends agree, and exactly where they do not.
//!
//! The crate offers "varying implementations to choose from", so what they do differently
//! is the thing a chooser needs to know, and it was recorded as one sentence: "regex does
//! not segment punctuation runs; charwalk does". That is true and too coarse to act on.
//! The two backends differ from charwalk in different ways, and from each other.
//!
//! These cases are the difference, written down. Every one was produced by running the
//! three against the same input, not from reading the code.

// The comparison needs something to compare against, and the default selection compiles
// only charwalk, so the cases below are gated on a regex backend being present.
#![cfg(any(feature = "use_regex", feature = "use_fancy_regex"))]

use word_bounds::impls::charwalk::Charwalk;
#[cfg(feature = "use_fancy_regex")]
use word_bounds::impls::fancy_regex::FancyRegex;
#[cfg(feature = "use_regex")]
use word_bounds::impls::regex::Regex;
use word_bounds::resolver::WordBoundResolver;
use word_bounds::rules::DefaultRules;

fn charwalk(input: &str) -> Vec<String> {
    WordBoundResolver::<Charwalk, DefaultRules>::resolve(input)
}

/// Inputs on which all three backends agree, which is most of what the crate is for.
const AGREED: &[&str] = &[
    "CamelCase",
    "snake_case",
    "kebab-case",
    "WordWithNumbers123",
    "thisExampleHasIDELikeACRONYMS",
    "UPPERCASELETTERS",
    "lowercaseletters",
    "a.b",
    "a.,b",
    "a!?!b",
];

/// A run is the same character repeated, and this is what says so.
///
/// It matters because it decides whether the `regex` backend could ever implement the
/// rule. Matching "the same character again" needs a backreference, which the `regex`
/// crate does not have by design; matching "more punctuation" would not. `a.,b` and
/// `a!?!b` are mixed punctuation and charwalk drops them, so the rule is the first thing
/// and the `regex` backend genuinely cannot express it.
#[test]
fn a_run_is_the_same_character_repeated_not_any_punctuation() {
    assert_eq!(
        charwalk("a...b"),
        ["a", "...", "b"],
        "repeated: kept as a run"
    );
    assert_eq!(charwalk("a..b"), ["a", "..", "b"]);
    assert_eq!(
        charwalk("a.,b"),
        ["a", "b"],
        "mixed: not a run, and dropped"
    );
    assert_eq!(charwalk("a.,.b"), ["a", "b"]);
    assert_eq!(charwalk("a!?!b"), ["a", "b"]);
}

#[cfg(feature = "use_regex")]
mod plain_regex {
    use super::*;

    fn under_test(input: &str) -> Vec<String> {
        WordBoundResolver::<Regex, DefaultRules>::resolve(input)
    }

    #[test]
    fn agrees_with_charwalk_wherever_runs_are_not_involved() {
        for input in AGREED {
            assert_eq!(under_test(input), charwalk(input), "disagreed on {input:?}");
        }
    }

    #[test]
    fn drops_an_interior_run() {
        // Charwalk gives ["a", "...", "b"].
        assert_eq!(under_test("a...b"), ["a", "b"]);
    }

    #[test]
    fn collapses_a_leading_or_trailing_run_to_one_character() {
        // The sharper half of the divergence, and the half the one-sentence note missed:
        // this backend does not merely fail to split a run, it loses all but one character
        // of it. Charwalk gives ["...", "leading"] and ["trailing", "..."].
        assert_eq!(under_test("...leading"), [".", "leading"]);
        assert_eq!(under_test("trailing..."), ["trailing", "."]);
    }
}

#[cfg(feature = "use_fancy_regex")]
mod fancy {
    use super::*;

    fn under_test(input: &str) -> Vec<String> {
        WordBoundResolver::<FancyRegex, DefaultRules>::resolve(input)
    }

    #[test]
    fn agrees_with_charwalk_wherever_runs_are_not_involved() {
        for input in AGREED {
            assert_eq!(under_test(input), charwalk(input), "disagreed on {input:?}");
        }
    }

    #[test]
    fn keeps_a_leading_or_trailing_run_whole() {
        // Where the plain regex backend loses all but one character, this one does not, so
        // the two are not the same defect and do not have the same fix.
        assert_eq!(under_test("...leading"), ["...", "leading"]);
        assert_eq!(under_test("trailing..."), ["trailing", "..."]);
    }

    #[test]
    fn drops_an_interior_run() {
        // The whole of what is left between this backend and charwalk. It is a removal
        // rather than a missing split: the run does not appear unsplit, it does not appear
        // at all, so the fix is in the post-pass and not in the pattern.
        assert_eq!(under_test("a...b"), ["a", "b"]);
        assert_eq!(
            under_test("...ellipses... could ... be hard..."),
            ["...", "ellipses", "could", "...", "be", "hard", "..."],
        );
    }
}

/// Many threads resolving at once, which is what found the race.
///
/// The compiled pattern is shared, and it used to be a `static mut` behind an `is_none()`
/// check: two threads both saw `None`, both built a regex, and one wrote the static while
/// the other read it. Rust's debug precondition checks caught the consequence inside
/// `regex-automata`, as `slice::get_unchecked requires that the index is within the slice`,
/// and aborted the process.
///
/// It was found by accident. Two of this crate's own tests happened to call the regex
/// backend at the same time, because `cargo test` runs tests in parallel, and the same
/// calls made one after another never showed it. This is that, on purpose.
#[test]
fn the_shared_pattern_survives_many_threads_arriving_together() {
    const THREADS: usize = 32;
    let expected = charwalk("ThisIsSomeRandom_text-to-split2");

    let outcomes: Vec<Vec<String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..THREADS)
            .map(|_| {
                scope.spawn(|| {
                    #[cfg(feature = "use_regex")]
                    let _ = WordBoundResolver::<Regex, DefaultRules>::resolve("a...b");
                    #[cfg(feature = "use_fancy_regex")]
                    let _ = WordBoundResolver::<FancyRegex, DefaultRules>::resolve("a...b");
                    charwalk("ThisIsSomeRandom_text-to-split2")
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("no thread panics"))
            .collect()
    });

    for outcome in &outcomes {
        assert_eq!(outcome, &expected, "a thread disagreed with the rest");
    }
}
