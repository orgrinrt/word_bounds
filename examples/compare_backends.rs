//! What each backend makes of the same input.
//!
//! The crate offers "varying implementations to choose from", so what they do differently
//! is the thing a chooser needs to know. This prints it.
//!
//! Run with: cargo run --example compare_backends --features use_regex,use_fancy_regex

use word_bounds::impls::charwalk::Charwalk;
#[cfg(feature = "use_fancy_regex")]
use word_bounds::impls::fancy_regex::FancyRegex;
#[cfg(feature = "use_regex")]
use word_bounds::impls::regex::Regex;
use word_bounds::resolver::WordBoundResolver;
use word_bounds::rules::DefaultRules;

const CASES: &[&str] = &[
    "a...b",
    "a.,b",
    "a.,.b",
    "a!?!b",
    "a..b",
    "a.b",
    "...leading",
    "trailing...",
    "...ellipses... could ... be hard...",
    "CamelCase",
    "snake_case",
    "kebab-case",
    "WordWithNumbers123",
];

fn main() {
    for input in CASES {
        println!("input: {input:?}");
        println!("  charwalk:    {:?}", WordBoundResolver::<Charwalk, DefaultRules>::resolve(input));
        #[cfg(feature = "use_regex")]
        println!("  regex:       {:?}", WordBoundResolver::<Regex, DefaultRules>::resolve(input));
        #[cfg(feature = "use_fancy_regex")]
        println!("  fancy_regex: {:?}", WordBoundResolver::<FancyRegex, DefaultRules>::resolve(input));
    }
}
