pub(crate) mod compiled;
pub mod charwalk;
#[cfg(any(feature = "use_fancy_regex", feature = "benchmark"))]
pub mod fancy_regex;
#[cfg(any(feature = "use_regex", feature = "benchmark"))]
pub mod regex;

#[macro_export]
macro_rules! __str_ext__instance_words_vec {
    ($s:expr, $vec:ident) => {
        #[cfg(feature = "optimize_for_cpu")]
        let mut $vec = Vec::with_capacity($s.len() / $crate::CHARS_PER_WORD_AVG);
        #[cfg(all(not(feature = "optimize_for_cpu"), feature = "optimize_for_memory"))]
        let mut $vec = Vec::with_capacity($s.len() / $crate::CHARS_PER_WORD_AVG as usize);
        #[cfg(all(not(feature = "optimize_for_cpu"), not(feature = "optimize_for_memory")))]
        let mut $vec = Vec::new();
    };
}

/// The compiled pattern, built once and shared.
///
/// A `OnceLock`, not a `static mut` behind an `is_none()` check.
///
/// The previous shape was a data race, and not a theoretical one: two threads calling
/// `resolve` both saw `None`, both built a regex, and one wrote the static while the other
/// was reading it. Rust's debug precondition checks caught the result inside
/// `regex-automata`, as `slice::get_unchecked requires that the index is within the
/// slice`, and aborted the process. It reproduced by running two of this crate's own tests
/// at once, which is what `cargo test` does by default; running the same calls one after
/// another never showed it.
///
/// `optimize_for_cpu` is on by default, so this was the default configuration.
///
/// `std::sync::OnceLock` rather than `once_cell`, which an earlier version of this fix
/// reached for. `once_cell` is optional here and gated on the two performance features, so
/// naming it unconditionally broke every build with a regex backend and neither flag. The
/// standard library's has been available since 1.70, which is what `rust-version` says.
#[macro_export]
macro_rules! __str_ext__cache_static_regex {
    ($regex:ty, $selfty:ty) => {
        #[cfg(not(feature = "optimize_for_memory"))]
        static REGEX: ::std::sync::OnceLock<$regex> = ::std::sync::OnceLock::new();

        /// Returns the shared pattern, compiling it on the first call.
        ///
        /// Two threads arriving together agree on the answer: one wins the initialisation
        /// and the other waits for it and sees the winner's value.
        #[cfg(not(feature = "optimize_for_memory"))]
        fn shared_regex<R>() -> &'static $regex
        where
            R: ResolverRules + 'static,
        {
            REGEX.get_or_init(|| match <$selfty>::compile_rules() {
                CompiledRules::Regex(r) => <$regex>::new(r.as_str())
                    .expect("Expected valid regex pattern"),
                _ => panic!("Compiled rules were not a Regex"),
            })
        }
    };
}

#[macro_export]
macro_rules! __str_ext__init_capture_iter {
    (plain $re_ident:ident, $regex:ty, $selfty:ty, $iter:ident, $s:expr) => {
        #[cfg(feature = "optimize_for_memory")]
        let $re_ident = match <$selfty>::compile_rules() {
            CompiledRules::Regex(r) => <$regex>::new(r.as_str()).expect(
                "Expected valid \
            regex pattern",
            ),
            _ => panic!("Compiled rules were not a Regex"),
        };

        #[cfg(not(feature = "optimize_for_memory"))]
        let $re_ident = shared_regex::<R>();
        #[cfg(not(feature = "optimize_for_memory"))]
        let $iter = $re_ident.captures_iter($s);
        #[cfg(feature = "optimize_for_memory")]
        let $iter = $re_ident.captures_iter($s);
    };
    (fancy $re_ident:ident, $regex:ty, $selfty:ty, $iter:ident, $s:expr) => {
        #[cfg(feature = "optimize_for_memory")]
        let $re_ident = match <$selfty>::compile_rules() {
            CompiledRules::Regex(r) => <$regex>::new(r.as_str()).expect(
                "Expected valid \
            fancy_regex pattern",
            ),
            _ => panic!("Compiled rules were not a Regex"),
        };

        // Since split function is not available in fancy_regex
        // we do it manually using find_iter
        #[cfg(not(feature = "optimize_for_memory"))]
        let $iter = shared_regex::<R>().find_iter($s);
        #[cfg(feature = "optimize_for_memory")]
        let $iter = $re_ident.find_iter($s);
    };
}
