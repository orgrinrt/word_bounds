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
        let mut raw = I::resolver(s);

        // raw = raw.into_iter().filter(|word| {
        //     if !R::special_chars_are_words() {
        //         return contains_special_chars(word);
        //     }
        //     false
        // }).collect();
        //
        // raw = raw.into_iter().map(|mut word| {
        //     if R::skip_prepended_underscores() {
        //         word = remove_prepended_underscores(word);
        //     }
        //     word
        // }).collect();

        raw
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

#[inline]
pub(crate) fn is_special_char(c: char) -> bool {
    c == '-' || c == '_'
}

#[inline]
pub(crate) fn contains_special_chars(word: &str) -> bool {
    word.chars().any(|c| is_special_char(c))
}

#[inline]
pub(crate) fn remove_prepended_underscores(word: String) -> String {
    if word.starts_with('_') {
        word.chars().skip_while(|&c| c == '_').collect()
    } else {
        word
    }
}
