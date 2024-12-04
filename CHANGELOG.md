<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

# Changelog

The latest published Color release is [0.1.0](#010-2024-11-20) which was released on 2024-11-20.
You can find its changes [documented below](#010-2024-11-20).

## [Unreleased]

This release has an [MSRV][] of 1.82.

### Added

* Add `BLACK`, `WHITE`, and `TRANSPARENT` constants to the color types. ([#64][] by [@waywardmonkeys][])
* The `serde` feature enables using `serde` with `AlphaColor`, `DynamicColor`, `OpaqueColor`, `PremulColor`, and `Rgba8`. ([#61][], [#70][] by [@waywardmonkeys][])
* Conversion of a `Rgba8` to a `u32` is now provided. ([#66][], [#77][] by [@waywardmonkeys][])
* A new `PremulRgba8` type mirrors `Rgba8`, but for `PremulColor`. ([#66][] by [@waywardmonkeys][])
* `AlphaColor::with_alpha` allows setting the alpha channel. ([#67][] by [@waywardmonkeys][])
* Support for the `ACEScg` colorspace. ([#54][] by [@MightyBurger][])
* `DynamicColor` gets `with_alpha` and `multiply_alpha`. ([#71][] by [@waywardmonkeys][])
* `DynamicColor` now impls `Hash` and `PartialEq`. ([#75][] by [@waywardmonkeys][])

### Changed

* The `mul_alpha` method was renamed to `multiply_alpha`. ([#65][] by [@waywardmonkeys][])
* Stray parenthesis in hex serialization of Rgba8 fixed ([#78][] by [@raphlinus][])

## [0.1.0][] (2024-11-20)

This release has an [MSRV][] of 1.82.

This is the initial release.

[@MightyBurger]: https://github.com/MightyBurger
[@raphlinus]: https://github.com/raphlinus
[@waywardmonkeys]: https://github.com/waywardmonkeys

[#54]: https://github.com/linebender/color/pull/54
[#61]: https://github.com/linebender/color/pull/61
[#64]: https://github.com/linebender/color/pull/64
[#65]: https://github.com/linebender/color/pull/65
[#66]: https://github.com/linebender/color/pull/66
[#67]: https://github.com/linebender/color/pull/67
[#70]: https://github.com/linebender/color/pull/70
[#71]: https://github.com/linebender/color/pull/71
[#75]: https://github.com/linebender/color/pull/75
[#77]: https://github.com/linebender/color/pull/77
[#78]: https://github.com/linebender/color/pull/78

[Unreleased]: https://github.com/linebender/color/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/linebender/color/releases/tag/v0.1.0

[MSRV]: README.md#minimum-supported-rust-version-msrv
