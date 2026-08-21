//! What the one compiled-in backend answers, when it is the only one there is.
//!
//! `backend_parity` is gated on a regex backend, and the default selection compiles only
//! the character walker, so under `cargo test` it reports `running 0 tests`. A suite
//! reporting nothing looks identical to one that stopped compiling and was never noticed,
//! which is a shape this crate has already shipped once.
//!
//! This file runs exactly when that one does not, so the pair always reports something.
//! What it reports has to be a fact about the crate rather than about itself: an earlier
//! version of this file bound a local to `1` and asserted it equalled `1`, which made the
//! count nonzero and could not fail for any value of anything.

#![cfg(not(any(feature = "use_regex", feature = "use_fancy_regex")))]

use word_bounds::impls::charwalk::Charwalk;
use word_bounds::resolver::WordBoundResolver;

/// The walker's answer, through the public resolver.
fn resolve(input: &str) -> Vec<String> {
    WordBoundResolver::<Charwalk>::resolve(input)
}

#[test]
fn the_character_walker_stands_alone() {
    // The backend that takes no dependency at all, and the only one `no_std` keeps. Under
    // this selection it is the whole crate, so these are the answers the crate gives.
    assert_eq!(
        resolve("someHTTPRequest_id"),
        ["some", "http", "request", "id"]
    );
    assert_eq!(resolve("camelCase"), ["camel", "case"]);
    assert_eq!(resolve("snake_case"), ["snake", "case"]);
    assert_eq!(resolve("kebab-case"), ["kebab", "case"]);
    assert_eq!(resolve("SCREAMING_SNAKE"), ["screaming", "snake"]);
    assert_eq!(resolve("Title Case"), ["title", "case"]);
}

#[test]
fn it_handles_what_the_regex_backends_do_not() {
    // The reason the walker is the default rather than a fallback: `## Known issues` in the
    // README records that the regex backends do not segment punctuation runs, and this one
    // does. A parity suite between backends cannot assert this, because on it they disagree,
    // which is why it belongs here rather than there.
    assert_eq!(resolve("a...b"), ["a", "...", "b"]);
}

#[test]
fn an_empty_input_yields_nothing() {
    assert!(resolve("").is_empty());
    assert!(resolve("   ").is_empty());
}
