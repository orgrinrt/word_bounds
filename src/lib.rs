use crate::rules::{DefaultRules, ResolverRules};

pub mod impls;
pub mod resolver;
pub mod rules;

#[cfg(feature = "optimize_for_cpu")]
pub(crate) const CHARS_PER_WORD_AVG: usize = 3;

#[cfg(all(not(feature = "optimize_for_cpu"), feature = "optimize_for_memory"))]
pub(crate) const CHARS_PER_WORD_AVG: u8 = 3;

pub trait WordBoundResolverImpl<R: ResolverRules = DefaultRules> {
    fn resolver(s: &str) -> Vec<String>;
    fn compile_rules() -> CompiledRules;
}

pub enum CompiledRules {
    Regex(String),
    Str(String),
    NotApplicable,
}
