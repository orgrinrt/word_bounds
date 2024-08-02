word_bounds
============
<div style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/word_bounds.svg)](https://github.com/orgrinrt/word_bounds/stargazers)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/word_bounds)](https://crates.io/crates/word_bounds)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/word_bounds.svg)](https://github.com/orgrinrt/word_bounds/issues)
[![Current Version](https://img.shields.io/badge/version-0.0.1-red.svg)](https://github.com/orgrinrt/word_bounds)

> Word bound detection and string segmentation with flexible rule-based approach and varying implementations to choose
> from

</div>

## Usage

`word_bounds` crate is intended to help detect word bounds and split up longer strings into smaller segments
based on rules that can be customized to fit your needs.

The rules allow flexible segmenting, for example, by either detecting chars as their own segments (words), bind them
together with the ongoing segment, or start the next segment with it. The rules also allow for removing or retaining
any chars, and has a customizable "sense" of punctuation chars (i.e you can detect words by underscores, whitespace,
etc.).

> Note: Work in progress; see [known issues](#known-issues) before choosing to use this crate

## Implementations & Performance

This repository currently contains three different methods to perform word bounds resolution - with standard `regex`
crate,
with `fancy_regex` crate, and a custom regexless char-walking version.

The performance of these methods is evaluated using `criterion`
benchmarking library. See [benches/segmentation.rs](benches/segmentation.rs) for the benchmarking code and
try it yourself. Here are the latest results on a macbook air m1 (which shows the relational performance, while the
exacts
will of course vary by system etc.):

| Trait                         | Execution Time       | Description                                                                                                                                                                                                                                                 |
|-------------------------------|----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `WordBoundResolverRegex`      | 119.09  µs (average) | ⚠️ **Major WIP** </br>(More) Accurate, but currently ~50x slower than `WordBoundResolverCharwalk`. Based on prior proof-of-concepts, we should ultimately land at around ~3x slower than the charwalk variant. Suitable for non-critical performance paths. |
| `WordBoundResolverFancyRegex` | 15.433  µs (average) | 🚧 **WIP, but taking shape** </br>All-inclusive regex logic including lookahead/lookback, which should be even more accurate, but ~7x slower than `WordBoundResolverCharwalk`. Use only when other variants fail.                                           |
| `WordBoundResolverCharwalk`   | 2.4 µs (average)     | ❎ **Somewhat complete; see: [known issues](#known-issues)** </br>Fastest and simplest, but could fail on certain edge cases. Officially suggested method for common cases.                                                                                  |

The `criterion` benchmark results show that `WordBoundResolverCharwalk` is the fastest, yet simplest, method, taking
only
about
2.4 µs on average per the benchmarking execution. The regex variants can be more accurate, and their logic is
using a tried and
tested framework, but they are significantly more expensive to run; the `WordBoundResolverRegex` that has no integrated
lookahead/lookback features, replaces this absence with a custom post-process pass, and should be about 3 times slower
than the
`WordBoundResolverCharwalk` variant (⚠️ *but is under construction, and while it passes the tests, it's 50x slower at
the moment* ⚠️). The
`WordBoundResolverFancyRegex` which makes use of the regex
engine for all of
its logic (including
lookahead/lookback), is more than 7 times slower than the `WordBoundResolverCharwalk` variant, though should yield
the most accurate results.

> Note: The regex variants are somewhat optimized, and in addition the crate has two different focuses for
> optimizations with
> the feature flags
`optimize_for_cpu` and
`optimize_for_memory`. These are not all that major differences, though the yields are *not insignificant*, and as
> such this is mostly relevant for someone
> doing extreme
> and
> picky
> optimizations on a
> larger project,
> otherwise one should stick to the defaults. The
> default configuration for optimizations bring the heaviest one, `fancy_regex` variant, down from around the 40 micro
> second range to its current ~15 micro second range (with the same system as for the above benchmark results). *Do
> note, though, that in general, optimising for memory here is fairly extreme, and makes the execution times
> exceedingly heavier by avoiding allocations outside of the stack.*

The official suggestion is to use `WordBoundResolverCharwalk` (i.e neither `use_regex`
nor `use_fancy_regex` features are enabled),
unless you face an edge case that isn't covered yet in the manual parsing logic. After that, you should test whether
`WordBoundResolverRegex` works, and if not, try `WordBoundResolverFancyRegex`.

> Note: Ultimately the costs are not usually all that significant, since this
> shouldn't be called in any hot loops, but your mileage may vary. Any and all issues and pull requests are welcome,
> if you face an edge case that isn't covered on the `WordBoundResolverCharwalk` variant.
>

## Known issues

### Maturity

A lot of the code is rough and naively implemented right now, some outright hacky, in order to reach
feature-completeness<span style="vertical-align: super; font-size: 0.5rem">1</span>. Things
are and
can be extremely messy, and it's probably not going to get better before the crate reaches the version 1.0 milestone
(feature-completeness).

Contributing, then, can be a headache. Sorry about that.

### Performance

It is clear that all of the implementations require further work in this regard. In prior proof-of-concepts, we were
able to reach execution times, for the charwalk method specifically, measured in the nanoseconds rather than
microseconds. The expansion and generalization of the rules made some of the decisions made back then infeasible,
and optimizations would have to be rethought almost entirely. Right now the focus has been finishing
the crate as a) feature-complete and b) well tested, and only afterwards find ways to decrease the running
costs<span style="vertical-align: super; font-size: 0.5rem">1</span>.

### Specification

Specification is currently only declared within the code, mainly in the unit tests (as explicit requirements to pass).
The rules themselves, that govern the behaviour of the segmentation, are not yet well-documented, and to make informed
choices
maintaining,
extending and
refactoring the project, a set collection of requirements will need to be documented within this repo. This is under
construction, but until this is done, contributing can be extra headache-inducing. Discussions on specifications is
more than welcome.

This also includes the public api, which isn't stable as yet. In general, a lot of the specification and documentation
side of this
crate remain
unfinished,
incomplete,
unfortunately.
Again, contributions, especially as
discussions, are more than welcome.

As it stands, the expanded tests with more challenging segmentation requirements are not passing for all of the
traits. Most of this is pretty straight-forward to implement (naively), but rules would have to be expanded to allow
flexibly
configuring this
kind of thing, which would then require a rework pass on all of the different segmentation methods. As a design
decision, it's not as straight-forward to choose how this would be designed to yield best
DX, and as such,
the
following
problems are  
currently WIP. Committing too much effort here towards a potentially misguided design that we'd have to rework later on
anyway, should be avoided. So, the specification needs to come first.

The currently known pain points that require further work, and are frozen until the specification work is done:

#### "Acronyms" of punctuation chars, such as ellipses as three periods

```
---- tests::test_word_bounds_charwalk stdout ----
thread 'tests::test_word_bounds_charwalk' panicked at tests/segmentation_short_strings.rs:99:13:
assertion `left == right` failed
  left: [".", "ellipses", "could", "be", "hard", "."]
 right: ["...", "ellipses", "...", "could", "...", "be", "hard", "..."]

---- tests::test_word_bounds_regex stdout ----
thread 'tests::test_word_bounds_regex' panicked at tests/segmentation_short_strings.rs:89:13:
assertion `left == right` failed
  left: [".", "ellipses", "could", "be", "hard", "."]
 right: ["...", "ellipses", "...", "could", "...", "be", "hard", "..."]

---- tests::test_word_bounds_fancy_regex stdout ----
thread 'tests::test_word_bounds_fancy_regex' panicked at tests/segmentation_short_strings.rs:78:13:
assertion `left == right` failed
  left: ["...", "ellipses", "could", "...", "be", "hard", "..."]
 right: ["...", "ellipses", "...", "could", "...", "be", "hard", "..."]

```

#### Modern unicode "chars", such as emojis

Charwalk is the only one that passes this test, but that doesn't guarantee that emojis work correctly with it right
now either. We need to expand the specifications (and as such, tests) to cover more extraordinary test scenarios.

```

---- tests::test_word_bounds_regex stdout ----
thread 'tests::test_word_bounds_regex' panicked at tests/segmentation_short_strings.rs:89:13:
assertion `left == right` failed
  left: ["maybe", "unicode", "emojis", "⚠", "are", "also", "🚧", "to", "be", "considered", "😅"]
 right: ["maybe", "unicode", "emojis", "⚠\u{fe0f}", "are", "also", "🚧", "to", "be", "considered", "😅"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_word_bounds_fancy_regex stdout ----
thread 'tests::test_word_bounds_fancy_regex' panicked at tests/segmentation_short_strings.rs:78:13:
assertion `left == right` failed
  left: ["maybe", "unicode", "emojis", "⚠", "\u{fe0f}", "are", "also", "🚧", "to", "be", "considered", "😅"]
 right: ["maybe", "unicode", "emojis", "⚠\u{fe0f}", "are", "also", "🚧", "to", "be", "considered", "😅"]

```

### Notes

<span style="vertical-align: super; font-size: 0.5rem">1</span><small> this crate's behaviour is required for a few of
the maintainer's other projects, which forces
this
prioritization right now. Be the change you want to see in the world, if this doesn't suit you.</small>

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying
me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> You can check out the full license [here](https://github.com/orgrinrt/word_bounds/blob/master/LICENSE)

This project is licensed under the terms of the **MIT** license.
