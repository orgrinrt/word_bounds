//! Segmenting into storage on the stack, with no allocator involved.
//!
//! The whole of this runs in two fixed arrays. Nothing here asks where memory came from,
//! which is the point: the same call takes a slice out of an arena or a region from an
//! allocator the caller already holds.
//!
//! ```text
//! cargo run --example lending --no-default-features --features no_alloc
//! ```

// `Outcome` is notko's, which is where the lending contract lives, and word_bounds
// re-exports it so nothing here needs a dependency on notko of its own.
use word_bounds::{fill_words, Outcome};

fn main() {
    println!("Segmenting into two stack arrays, no allocator anywhere.\n");

    // The characters of every word, run together, and one pair per word saying where each
    // begins and ends inside them.
    let mut text = [0u8; 128];
    let mut bounds = [(0usize, 0usize); 16];

    for input in [
        "someHTTPRequest_id",
        "parse_XMLDocument",
        "kebab-case-and-more",
        "digits123mixed456",
    ] {
        let words = fill_words(input, &mut text, &mut bounds)
            .expect("128 bytes and 16 bounds is enough for these");

        print!("{input:<24}");
        for word in words.iter() {
            print!(" [{word}]");
        }
        println!();
    }

    println!("\nA lend too small says how much it wanted, rather than truncating.\n");

    // Two words' worth of bounds, against an input with four words in it.
    let mut small_bounds = [(0usize, 0usize); 2];
    match fill_words("someHTTPRequest_id", &mut text, &mut small_bounds) {
        Outcome::Ok(words) => println!("unexpectedly fitted {} words", words.len()),
        Outcome::Err(exhausted) => println!(
            "refused: wanted at least {}, had {}",
            exhausted.wanted, exhausted.had
        ),
    }

    // The same for the text lend. Doubling from `wanted` converges, which is what carrying
    // both numbers is for.
    let mut small_text = [0u8; 4];
    match fill_words("someHTTPRequest_id", &mut small_text, &mut bounds) {
        Outcome::Ok(words) => println!("unexpectedly fitted {} bytes", words.text_len()),
        Outcome::Err(exhausted) => println!(
            "refused: wanted at least {} bytes, had {}",
            exhausted.wanted, exhausted.had
        ),
    }

    println!("\nA word ends where the walker says, and the lowercasing follows it.\n");

    // A capital sigma at the end of a word takes the final form, which is the one place
    // lowercasing each character on its own would have given a different answer.
    let mut greek_text = [0u8; 64];
    let mut greek_bounds = [(0usize, 0usize); 8];
    let greek = fill_words("ΟΔΟΣ_ΣΤΟ", &mut greek_text, &mut greek_bounds).unwrap();

    print!("{:<24}", "ΟΔΟΣ_ΣΤΟ");
    for word in greek.iter() {
        print!(" [{word}]");
    }
    println!("  (final sigma in the first, ordinary in the second)");
}
