# `word_bounds`

> Word-bound detection and string segmentation, rule-based, with several implementations to pick from.

`word_bounds` finds the bounds between words and splits longer strings into segments by rules you can customise. A rule can make a character a segment of its own, bind it to the segment in progress or start the next segment with it, characters can be removed or kept, and the crate's sense of punctuation is configurable, so words can be found by underscores, whitespace or whatever the input uses. `camelCase`, `snake_case`, `kebab-case`, `SCREAMING_SNAKE` and `Title Case` all come apart into their words.

## Usage

Not published to crates.io yet, so it is a git dependency for now.

```toml
[dependencies]
word_bounds = { git = "https://github.com/orgrinrt/word_bounds", branch = "dev" }
```

There are three implementations under `word_bounds::impls`. `charwalk::Charwalk` is a regexless walk over the characters, it is the default and the one to use unless you hit a case it doesn't cover, and it is the only one that reads a run of the same punctuation character as one token, so `...` is one token. `regex::Regex` and `fancy_regex::FancyRegex` come with the `use_regex` and `use_fancy_regex` features. `cargo run --example compare_backends --features use_regex,use_fancy_regex` prints what each makes of the same input, and `cargo bench --features benchmark` measures them with criterion, where `Charwalk` comes out fastest by a wide margin.

## Allocation

The default build has everything, including the regex backends and `Vec<String>` output. `no_std` leaves `Charwalk` and `alloc`, since both regex crates need `std`, and `no_alloc` implies `no_std` and adds `fill_words`, which writes into storage the caller lends and allocates nothing. It doesn't take the allocating api away, so a crate with an allocator can enable it for one hot path and keep both. `tests/feature_matrix.rs` builds each of the three.

```rust
use word_bounds::fill_words;

let mut text = [0u8; 32];
let mut bounds = [(0, 0); 8];

let words = fill_words("someHTTPRequest_id", &mut text, &mut bounds).unwrap();

assert_eq!(words.len(), 4);
assert_eq!(words.get(0), Some("some"));
assert_eq!(words.get(3), Some("id"));
```

A lend that is too small refuses and reports how much was wanted against how much there was. Both apis go through one walk, so they can't disagree about where a word ends, and `tests/lending_parity.rs` compares them on 31 inputs for lowercasing, where they could.

## Known issues

A lot of the code is rough, written to reach feature-completeness first, and it will probably stay that way until 1.0. Everything is tested against the default rules, so the rule system is neither stable nor actively tested, which limits it for now.

The regex backends don't segment punctuation runs. Their tests state the intended behaviour and are marked `#[ignore]` with a catalogue reason, and `cargo test --features use_regex,use_fancy_regex -- --ignored` is expected to fail and shows what is missing. The `regex::Regex` backend is also still under construction and some of its tests fail. On speed, the walk was once measured in nanoseconds and generalising the rules gave that up, some of it has been won back by reducing the rules once per input, and the rest means rethinking the arrangement.

## License

> The project is licensed under the **Mozilla Public License 2.0**.

`SPDX-License-Identifier: MPL-2.0`

> You can check out the full license [here](https://github.com/orgrinrt/word_bounds/blob/dev/LICENSE)
