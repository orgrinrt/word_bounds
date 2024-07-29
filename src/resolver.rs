use std::marker::PhantomData;

use crate::impls::charwalk::Charwalk;
use crate::rules::{DefaultRules, ResolverRules};
use crate::WordBoundResolverImpl;

pub struct WordBoundResolver<
    'a,
    I: WordBoundResolverImpl<R> = Charwalk,
    R: ResolverRules = DefaultRules,
> {
    _phantom_data: PhantomData<(I, R)>,
    input: &'a str,
    words: Vec<String>,
}

impl<'a, I: WordBoundResolverImpl<R>, R: ResolverRules> WordBoundResolver<'a, I, R> {
    pub fn new(s: &'a str) -> Self {
        Self {
            _phantom_data: PhantomData::default(),
            input: s,
            words: Vec::new(),
        }
    }
}

impl<'a, I: WordBoundResolverImpl<R>, R: ResolverRules> WordBoundResolver<'a, I, R> {
    pub fn resolve(s: &str) -> Vec<String> {
        I::resolver(s)
    }

    #[inline]
    pub fn resolve_with<I2: WordBoundResolverImpl<R2>, R2: ResolverRules>(s: &str) -> Vec<String> {
        I2::resolver(s)
    }

    #[inline]
    pub fn run(mut self) -> Self {
        self.words = Self::resolve(self.input);
        self
    }
}
