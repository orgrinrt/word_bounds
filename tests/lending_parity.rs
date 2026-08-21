//! The lending API answers the same as the allocating one, or it is not the same API.
//!
//! Both go through one walk, so the segmentation cannot differ. What can differ is the
//! lowercasing: the allocating sink calls `str::to_lowercase` on a whole word, and the
//! lending one cannot, because it has nowhere to hold the word while it does. It maps each
//! character instead and reproduces by hand the one rule the two disagree on, which is that
//! a capital sigma ending a word takes the final form.
//!
//! That reproduction is the thing worth pinning. Every case below is run through both and
//! the answers compared, so a divergence shows up as a failure here rather than as a wrong
//! word in somebody's output.

#![cfg(feature = "no_alloc")]

use word_bounds::fill_words;
use word_bounds::impls::charwalk::Charwalk;
use word_bounds::sink::is_cased;
use word_bounds::resolver::WordBoundResolver;

/// The allocating API's answer, through the backend `fill_words` also uses.
///
/// Named rather than left to inference: `WordBoundResolver` defaults its backend, and a
/// default is only reached when nothing else pins it, which an associated function call
/// does not do.
fn via_alloc(input: &str) -> Vec<String> {
    WordBoundResolver::<Charwalk>::resolve(input)
}

/// Everything both APIs are asked, in one place.
///
/// Add a case here and both directions get it.
const CASES: &[&str] = &[
    // The ordinary shapes.
    "someHTTPRequest_id",
    "snake_case_name",
    "camelCaseName",
    "PascalCaseName",
    "kebab-case-name",
    "SCREAMING_SNAKE",
    "Title Case Words",
    "mixed_Case-with.punctuation",
    "trailing_",
    "_leading",
    "double__underscore",
    "digits123mixed456",
    "123",
    "a",
    "",
    "   ",
    "...",
    "a1b2c3",
    "XMLHttpRequest",
    "IOError",
    // Where the two lowercasings could part company. A capital sigma ending a word
    // lowercases to the final form; anywhere else it is the ordinary one.
    "ΟΔΟΣ",
    "ΣΙΓΜΑ",
    "ΑΣ ΣΑΣ",
    "Σ",
    "ΟΔΟΣ_ΣΤΟ",
    "camelΣCase",
    "ΤΕΛΟΣ ΑΡΧΗ",
    // The three shapes a reviewer found the reproduction was missing. Each is a way the
    // real rule differs from "look at the character immediately before the sigma".
    //
    // A combining accent between the letter and the sigma. It is `Case_Ignorable`, so
    // `str::to_lowercase` skips it walking backwards and still finds the alpha.
    "Α\u{301}Σ",
    "ΑΒ\u{301}\u{308}Σ",
    // A soft hyphen, which is `Cf` and also `Case_Ignorable`.
    "Α\u{ad}Σ",
    // A titlecase letter, which is `Cased` and answers false to both `is_lowercase` and
    // `is_uppercase`.
    "ǅΣ",
    "ǄΣ",
    "ǈΣ",
    // Something genuinely not cased before the sigma, which is the case that must still
    // come out as the ordinary form.
    "1Σ",
    "\u{301}Σ",
    // Non-ASCII that is not Greek, since the ASCII fast paths in the walker must not
    // change the answer for anything else.
    "ÄÖÜ_äöü",
    "naïveRésumé",
    "ЗдравствуйМир",
    "日本語テキスト",
    "emoji🙂here",
];

/// The lending API's answer, as owned strings so it can be compared.
fn via_lending(input: &str) -> Vec<String> {
    // Generous, so nothing here is testing the refusal path by accident. The refusal has
    // its own tests.
    let mut text = [0u8; 512];
    let mut bounds = [(0usize, 0usize); 64];

    let segmented = fill_words(input, &mut text, &mut bounds)
        .unwrap_or_else(|e| panic!("{input:?} did not fit: wanted {}, had {}", e.wanted, e.had));

    segmented.iter().map(str::to_owned).collect()
}

#[test]
fn both_apis_find_the_same_words() {
    for &input in CASES {
        let allocating = via_alloc(input);
        let lending = via_lending(input);

        assert_eq!(
            allocating, lending,
            "{input:?}: allocating gave {allocating:?}, lending gave {lending:?}",
        );
    }
}

#[test]
fn the_final_sigma_rule_is_reproduced_rather_than_approximated() {
    // The control for the test above: without this case the parity assertion would pass
    // over a mapping that never applies the rule, because every other case has the same
    // answer either way.
    //
    // `str::to_lowercase` puts the final form at the end of a word and the ordinary form
    // everywhere else, and this is what that difference looks like.
    assert_eq!("ΟΔΟΣ".to_lowercase(), "οδος");
    assert_ne!("ΟΔΟΣ".to_lowercase(), "οδοσ");

    // Which is what mapping each character on its own would have given, so the two really
    // do disagree and the reproduction is doing work.
    let per_char: String = "ΟΔΟΣ".chars().flat_map(char::to_lowercase).collect();
    assert_eq!(per_char, "οδοσ");
    assert_ne!(per_char, "ΟΔΟΣ".to_lowercase());

    // And the lending API takes the first answer rather than the second.
    assert_eq!(via_lending("ΟΔΟΣ"), vec!["οδος".to_string()]);
}

#[test]
fn the_rule_looks_past_an_ignorable_character_for_a_cased_one() {
    // `str::to_lowercase` walks backwards past `Case_Ignorable` characters looking for a
    // cased one. Looking only at the character immediately before the sigma is what the
    // sink used to do, and it gave the ordinary form here where the allocating path gives
    // the final one.
    //
    // Tracking the most recent cased character rather than the immediately preceding one
    // performs the skip exactly, because an ignorable character never updates it.
    assert_eq!("Α\u{301}Σ".to_lowercase(), "α\u{301}ς");
    assert_eq!(via_lending("Α\u{301}Σ"), vec!["α\u{301}ς".to_string()]);

    assert_eq!("Α\u{ad}Σ".to_lowercase(), "α\u{ad}ς");
    assert_eq!(via_lending("Α\u{ad}Σ"), vec!["α\u{ad}ς".to_string()]);
}

#[test]
fn a_titlecase_letter_counts_as_cased() {
    // Unicode `Cased` is `Lowercase` or `Uppercase` or general category `Lt`. `ǅ` is `Lt`
    // and answers false to `char::is_lowercase` and `char::is_uppercase` alike, so a sigma
    // after it was treated as though nothing cased preceded it.
    assert!(!'ǅ'.is_lowercase());
    assert!(!'ǅ'.is_uppercase());
    assert!(is_cased('ǅ'), "titlecase is cased");

    assert_eq!("ǅΣ".to_lowercase(), "ǆς");
    assert_eq!(via_lending("ǅΣ"), vec!["ǆς".to_string()]);
}

#[test]
fn something_uncased_before_the_sigma_leaves_the_ordinary_form() {
    // The control for the two above. Without it they would pass just as well against an
    // `is_cased` that answered true for everything, which would make every trailing sigma
    // final.
    assert!(!is_cased('1'));
    assert!(!is_cased('\u{301}'));

    assert_eq!(via_lending("1Σ"), via_alloc("1Σ"));
    assert_eq!(via_lending("\u{301}Σ"), via_alloc("\u{301}Σ"));
}

#[test]
fn a_sigma_that_is_not_final_keeps_the_ordinary_form() {
    // The other half of the rule. A sigma in the middle of a word is not final, and one
    // with nothing cased before it is not either.
    assert_eq!(via_lending("ΣΙΓΜΑ"), via_alloc("ΣΙΓΜΑ"));
    assert_eq!(via_lending("Σ"), via_alloc("Σ"));

    // A lone sigma has no cased letter before it, so it is not final: `σ`, not `ς`.
    assert_eq!(via_lending("Σ"), vec!["σ".to_string()]);
}

#[test]
fn a_word_that_ends_at_a_bound_still_gets_the_rule() {
    // The walker ends a word at a case change or a delimiter rather than only at the end of
    // the input, and the sigma rule has to fire at each of those too. `str::to_lowercase`
    // sees each word on its own, so it does; the lending sink has to be told a word ended.
    for input in ["ΟΔΟΣ_ΣΤΟ", "ΑΣ ΣΑΣ", "ΤΕΛΟΣ ΑΡΧΗ"] {
        assert_eq!(via_lending(input), via_alloc(input), "{input:?}");
    }
}

#[test]
fn the_text_lend_holds_exactly_the_words_run_together() {
    let mut text = [0u8; 64];
    let mut bounds = [(0usize, 0usize); 8];
    let segmented = fill_words("someHTTPRequest_id", &mut text, &mut bounds).unwrap();

    // Four words, no separator between them, so the text length is the sum of their
    // lengths and the bounds partition it exactly.
    assert_eq!(segmented.len(), 4);
    assert_eq!(segmented.text_len(), "somehttprequestid".len());

    let mut at = 0;
    for word in segmented.iter() {
        at += word.len();
    }
    assert_eq!(at, segmented.text_len(), "the bounds leave a gap or overlap");
}
