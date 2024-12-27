<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

# Changelog

The latest published Color release is [0.2.1](#021-2024-12-27) which was released on 2024-12-27.
You can find its changes [documented below](#021-2024-12-27).

## [Unreleased]

This release has an [MSRV][] of 1.82.

## [0.2.1][] (2024-12-27)

This release has an [MSRV][] of 1.82.

### Added

* Add `FromStr` impl for `AlphaColor`, `DynamicColor`, `OpaqueColor`, `PremulColor`. ([#111][] by [@waywardmonkeys][])

### Changed

* Don't enable `serde`'s `std` feature when enabling our `std` feature. ([#108][] by [@waywardmonkeys][])
* `From<Rgba8>` for `PremulColor` is deprecated and replaced by `From<PremulRgba8>`. ([#113][] by [@waywardmonkeys][])

### Fixed

* Make color parsing case insensitive. ([#109][] by [@raphlinus][])

## [0.2.0][] (2024-12-17)

This release has an [MSRV][] of 1.82.

### Added

* Add `BLACK`, `WHITE`, and `TRANSPARENT` constants to the color types. ([#64][] by [@waywardmonkeys][])
* The `serde` feature enables using `serde` with `AlphaColor`, `DynamicColor`, `HueDirection`, `OpaqueColor`, `PremulColor`, and `Rgba8`. ([#61][], [#70][], [#80][] by [@waywardmonkeys][])
* Conversion of a `Rgba8` to a `u32` is now provided. ([#66][], [#77][] by [@waywardmonkeys][], [#100][] by [@tomcur][])
* A new `PremulRgba8` type mirrors `Rgba8`, but for `PremulColor`. ([#66][] by [@waywardmonkeys][])
* `AlphaColor::with_alpha` allows setting the alpha channel. ([#67][] by [@waywardmonkeys][])
* Support for the `ACEScg` color space. ([#54][] by [@MightyBurger][])
* `DynamicColor` gets `with_alpha` and `multiply_alpha`. ([#71][] by [@waywardmonkeys][])
* `DynamicColor` now impls `PartialEq`. ([#75][] by [@waywardmonkeys][])
* `AlphaColor`, `OpaqueColor`, and `PremulColor` now impl `PartialEq`. ([#76][], [#86][] by [@waywardmonkeys][])
* `HueDirection` now impls `PartialEq`. ([#79][] by [@waywardmonkeys][])
* `ColorSpaceTag` and `HueDirection` now have bytemuck support. ([#81][] by [@waywardmonkeys][])
* A `DynamicColor` parsed from a named color or named color space function now serializes back to that name, as per the CSS Color Level 4 spec ([#39][] by [@tomcur][]).
* `CacheKey` to allow using colors as keys for resource caching. ([#92][] by [@DJMcNab][])

### Changed

* The `mul_alpha` method was renamed to `multiply_alpha`. ([#65][] by [@waywardmonkeys][])

### Fixed

* Stray parenthesis in hex serialization of `Rgba8` fixed. ([#78][] by [@raphlinus][])

## [0.1.0][] (2024-11-20)

This release has an [MSRV][] of 1.82.

This is the initial release.

[@DJMcNab]: https://github.com/DJMcNab
[@MightyBurger]: https://github.com/MightyBurger
[@raphlinus]: https://github.com/raphlinus
[@tomcur]: https://github.com/tomcur
[@waywardmonkeys]: https://github.com/waywardmonkeys

[#39]: https://github.com/linebender/color/pull/39
[#54]: https://github.com/linebender/color/pull/54
[#61]: https://github.com/linebender/color/pull/61
[#64]: https://github.com/linebender/color/pull/64
[#65]: https://github.com/linebender/color/pull/65
[#66]: https://github.com/linebender/color/pull/66
[#67]: https://github.com/linebender/color/pull/67
[#70]: https://github.com/linebender/color/pull/70
[#71]: https://github.com/linebender/color/pull/71
[#75]: https://github.com/linebender/color/pull/75
[#76]: https://github.com/linebender/color/pull/76
[#77]: https://github.com/linebender/color/pull/77
[#78]: https://github.com/linebender/color/pull/78
[#79]: https://github.com/linebender/color/pull/79
[#80]: https://github.com/linebender/color/pull/80
[#81]: https://github.com/linebender/color/pull/81
[#86]: https://github.com/linebender/color/pull/86
[#92]: https://github.com/linebender/color/pull/92
[#100]: https://github.com/linebender/color/pull/100
[#108]: https://github.com/linebender/color/pull/108
[#109]: https://github.com/linebender/color/pull/109
[#111]: https://github.com/linebender/color/pull/111
[#113]: https://github.com/linebender/color/pull/113

[Unreleased]: https://github.com/linebender/color/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/linebender/color/releases/tag/v0.2.1
[0.2.0]: https://github.com/linebender/color/releases/tag/v0.2.0
[0.1.0]: https://github.com/linebender/color/releases/tag/v0.1.0

[MSRV]: README.md#minimum-supported-rust-version-msrv
