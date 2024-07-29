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

#[macro_export]
macro_rules! __str_ext__cache_static_regex {
    ($regex:ty, $selfty:ty) => {
        #[cfg(not(feature = "optimize_for_memory"))]
        static mut REGEX: Option<$regex> = None;

        #[cfg(not(feature = "optimize_for_memory"))]
        unsafe fn set_regex<R>()
        where
            R: ResolverRules + 'static,
        {
            let new_re = match <$selfty>::compile_rules() {
                CompiledRules::Regex(r) => <$regex>::new(r.as_str()).expect(
                    "Expected valid \
                fancy_regex pattern",
                ),
                _ => panic!("Compiled rules were not a Regex"),
            };

            REGEX = Some(new_re);
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
        let $re_ident = unsafe {
            if REGEX.is_none() {
                set_regex::<R>()
            }
            REGEX.as_ref().unwrap()
        };
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
        let $iter = unsafe {
            if REGEX.is_none() {
                set_regex::<R>()
            }
            REGEX.as_ref().unwrap().find_iter($s)
        };
        #[cfg(feature = "optimize_for_memory")]
        let $iter = $re_ident.find_iter($s);
    };
}
