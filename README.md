word_bounds
============
<div style="text-align: center;">

[![GitHub Stars](https://img.shields.io/github/stars/orgrinrt/word_bounds.svg)](https://github.com/orgrinrt/word_bounds/stargazers)
[![GitHub Issues](https://img.shields.io/github/issues/orgrinrt/word_bounds.svg)](https://github.com/orgrinrt/word_bounds/issues)
[![Current Version](https://img.shields.io/badge/version-0.0.2-red.svg)](https://github.com/orgrinrt/word_bounds)

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

This repository currently contains three different methods to perform word bounds resolution: with the standard `regex`
crate,
with `fancy_regex` crate, and a custom regexless char-walking version.

The performance of these methods is evaluated using `criterion`
benchmarking library. See [benches/segmentation.rs](benches/segmentation.rs) for the benchmarking code and
try it yourself with `cargo bench --features benchmark`. The bench measures all three implementations, so it
needs that feature to compile: the two regex ones are behind feature gates and are absent from a default
build. Here are the latest results on a macbook air m1 (which shows the relational performance, while the
exacts
will of course vary by system etc.):

| Implementation (`word_bounds::impls::*`) | Execution Time       | Description                                                                                                                                                                                                                                                 |
|-------------------------------------------|----------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `regex::Regex`      | 119.09  µs (average) | **Major WIP** </br>(More) Accurate, but currently ~50x slower than `charwalk::Charwalk`. Based on prior proof-of-concepts, we should ultimately land at around ~3x slower than the charwalk variant. Suitable for non-critical performance paths. |
| `fancy_regex::FancyRegex` | 15.433  µs (average) | **WIP, but taking shape** </br>All-inclusive regex logic including lookahead/lookback, which should be even more accurate, but ~7x slower than `charwalk::Charwalk`. Use only when other variants fail.                                           |
| `charwalk::Charwalk`   | 2.4 µs (average)     | **Passes the current segmentation suite; see: [known issues](#known-issues)** </br>Fastest and simplest, and the only implementation that handles punctuation runs and variation-selector emoji. Officially suggested method.                                                                                  |

The `criterion` benchmark results show that `charwalk::Charwalk` is the fastest, yet simplest, method, taking
only
about
2.4 µs on average per the benchmarking execution.

> Note: the figures in the table were measured before the rules were compiled once per input
> rather than searched per character. On the same input and machine that change and the ones with
> it cut `charwalk::Charwalk` to about half of what it was, so its row reads high. The regex rows
> are unaffected: they do not read the rules the same way. The regex variants can be more accurate, and their logic is
using a tried and
tested framework, but they are significantly more expensive to run; the `regex::Regex` implementation, which has no integrated
lookahead/lookback features, replaces this absence with a custom post-process pass, and should be about 3 times slower
than the
`charwalk::Charwalk` variant (*but is under construction, and some of its tests currently fail; see [known issues](#known-issues)*). The
`fancy_regex::FancyRegex` implementation, which makes use of the regex
engine for all of
its logic (including
lookahead/lookback), is more than 7 times slower than the `charwalk::Charwalk` variant, though should yield
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

The official suggestion is to use `charwalk::Charwalk` (i.e neither `use_regex`
nor `use_fancy_regex` features are enabled),
unless you face an edge case that isn't covered yet in the manual parsing logic. After that, you should test whether
`regex::Regex` works, and if not, try `fancy_regex::FancyRegex`.

> Note: Ultimately the costs are not usually all that significant, since this
> shouldn't be called in any hot loops, but your mileage may vary. Any and all issues and pull requests are welcome,
> if you face an edge case that isn't covered on the `charwalk::Charwalk` variant.
>

## Known issues

### Maturity

A lot of the code is rough and naively implemented right now, some outright hacky, in order to reach
feature-completeness<sup>1</sup>. Things
are and
can be extremely messy, and it's probably not going to get better before the crate reaches the version 1.0 milestone
(feature-completeness).

Contributing, then, can be a headache. Sorry about that.

In addition, everything is currently tested against the default rules, which means that the rule system is not
currently stable or even actively tested. This limits the usability quite a bit for now.

### The regex backends do not segment punctuation runs

`charwalk::Charwalk` reads a run of the same punctuation character as one token, so `...` is a
single token rather than three. Neither regex backend does. `regex` has no backreferences by
design, and while `fancy_regex` has them, the rule is not expressed in its pattern either; both
say so at their `RuleTarget::PunctSpecialCharRun` arm.

The segmentation tests for those two backends state the intended behaviour and are marked
`#[ignore]` with a catalogue reason rather than weakened to match what the backends do, so a normal
run stays green while the gap is visible:

```bash
cargo test --features use_regex,use_fancy_regex -- --ignored
```

That command is expected to fail, and the failure is the specification of what is missing.

### Performance

In prior proof-of-concepts the charwalk method reached execution times measured in nanoseconds rather
than microseconds. Generalising the rules made the decisions behind those numbers infeasible, and
getting back there means rethinking the arrangement rather than tightening the loop.

Some of that is done. The rules are now reduced once per input to the flags the walk reads, rather
than the walk searching the rule list for every character and every rule kind; character
membership is a bit test rather than a scan of a string; the default set of special characters is
built without a hash set; and the walk makes one pass, so neither the characters nor the runs of
punctuation are collected up front. Together those roughly halved the time per call.

What remains is mostly allocation, and it is measurable: about a sixth of a call goes on the three
rule methods, which each allocate on every call because their signatures return owned values, and
most of the rest is one `String` per word, which is what returning `Vec<String>` asks for. Getting
to nanoseconds means the segmentation handing back slices of the input rather than owned strings,
and the rules being compiled once per ruleset rather than once per input. Both change the public
surface, so they are design decisions rather than optimisations.

Right now the focus has been finishing
the crate as a) feature-complete and b) well tested, and only afterwards find ways to decrease the running
costs<sup>1</sup>

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

The expanded tests with the more challenging segmentation requirements now pass for `charwalk`,
the default and suggested implementation. Getting there meant extending the rule system rather
than special-casing the input: punctuation runs are their own rule target, so the behaviour is
configurable through the same mechanism as everything else, and an implementation that does not
support them says so instead of guessing.

The same case is still open for the two regex implementations, and the caution that applied before
still applies to them: a naive fix is easy to write and easy to get wrong in a way that has to be
reworked once the specification catches up. The specification work is what unblocks doing this
properly across all three.

The currently known pain points that require further work:

#### "Acronyms" of punctuation chars, such as ellipses as three periods

A run of two or more of the same punctuation character reads as one token rather than as separate
punctuation: `...` is a unit, while a lone `.` between words is a separator. This is the
`PunctSpecialCharRun` rule target, and the default rules bound a word on each side of a run, so a
run survives the `Remove(PunctSpecialChar, Middle(..))` that strips a lone separator.

`charwalk` implements it and passes the segmentation tests. The other two do not:

- `regex` cannot express it. Matching a run of the *same* character needs a backreference, and the
  `regex` crate has none by design. Supporting runs there needs a different approach, such as
  merging adjacent identical punctuation tokens after the split.
- `fancy_regex` has the backreferences to express it, but the rule is not written into its pattern
  yet. It names the target so rule compilation stays total, and otherwise ignores it.

#### Modern unicode "chars", such as emojis

`charwalk` passes this test, including the variation-selector case (`⚠️` is `U+26A0 U+FE0F`, which
has to stay one token). That is not a guarantee that emoji are handled correctly in general; the
specification and the tests need to cover more of these before that can be claimed.

The other two implementations fail here. `regex` drops the variation selector, and `fancy_regex`
splits it into its own token:

```text
---- tests::test_word_bounds_regex stdout ----
  left: ["maybe", "unicode", "emojis", "⚠", "are", "also", "🚧", "to", "be", "considered", "😅"]
 right: ["maybe", "unicode", "emojis", "⚠\u{fe0f}", "are", "also", "🚧", "to", "be", "considered", "😅"]

---- tests::test_word_bounds_fancy_regex stdout ----
  left: ["maybe", "unicode", "emojis", "⚠", "\u{fe0f}", "are", "also", "🚧", "to", "be", "considered", "😅"]
 right: ["maybe", "unicode", "emojis", "⚠\u{fe0f}", "are", "also", "🚧", "to", "be", "considered", "😅"]
```

### Notes

<sup>1</sup><small> this crate's behaviour is required for a few of
the maintainer's other projects, which forces
this
prioritization right now. Be the change you want to see in the world, if this doesn't suit you.</small>

## Support

Whether you use this project, have learned something from it, or just like it, please consider supporting it by buying
me a coffee, so I can dedicate more time on open-source projects like this :)

<a href="https://buymeacoffee.com/orgrinrt" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: auto !important;width: auto !important;" ></a>

## License

> You can check out the full license [here](https://github.com/orgrinrt/word_bounds/blob/main/LICENSE)

This project is licensed under the terms of the **MPL-2.0** license.
