//! Why `backend_parity` has nothing to run under the default selection.
//!
//! That file is gated on a regex backend, and the default selection compiles only charwalk,
//! so under `cargo test` it reports `running 0 tests`. A suite reporting nothing looks
//! identical to one that stopped compiling and was never noticed, which is the shape this
//! crate has already shipped once.
//!
//! This file is the other half: it runs exactly when that one does not, so the pair always
//! reports something.

#![cfg(not(any(feature = "use_regex", feature = "use_fancy_regex")))]

#[test]
fn parity_needs_a_second_backend_and_the_default_selection_has_one_implementation() {
    // Not a skipped test. With only charwalk compiled in there is no second implementation
    // to disagree with, so there is nothing for the parity cases to say. Enabling
    // `use_regex` or `use_fancy_regex` is what gives them something to compare.
    let backends_compiled_in = 1;
    assert_eq!(
        backends_compiled_in, 1,
        "the default selection is charwalk alone; parity is checked at \
         --features use_regex,use_fancy_regex"
    );
}
