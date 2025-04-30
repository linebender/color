<!-- Instructions

This changelog follows the patterns described here: <https://keepachangelog.com/en/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

-->

# Changelog

The latest published Color release is [0.3.0](#030-2025-04-30) which was released on 2025-04-30.
You can find its changes [documented below](#030-2025-04-30).

## [Unreleased]

This release has an [MSRV][] of 1.82.

## [0.3.0][] (2025-04-30)

This release has an [MSRV][] of 1.82.

### Added

* Support converting between color spaces without chromatic adaptation, thereby representing the same absolute color in the destination color space as in the source color space. ([#139][], [#153][] by [@tomcur][])
  * Add absolute color conversion matrices for ProPhoto RGB, ACES2065-1 and ACEScg for faster conversion without chromatic adaptation to and from these color spaces. ([#156][], [#164][], [#165][] by [@tomcur][])

  **Note to `ColorSpace` implementers:** the `WHITE_POINT` associated constant is added to `ColorSpace`, defaulting to D65.
  Implementations with a non-D65 white point should set this constant to get correct default absolute conversion behavior.
* Support manual chromatic adaptation of colors between arbitrary white point chromaticities.  ([#139][] by [@tomcur][])
* Add `Missing::EMPTY` to allow getting an empty `Missing` set in `const` contexts. ([#149][] by [@tomcur][])
* Add `From<AlphaColor<_>> for DynamicColor` conversions for all color spaces that have a direct runtime representation in `ColorSpaceTag`. ([#155][] by [@LaurenzV][])
* Add usage examples for `DynamicColor::interpolate` and `gradient`. ([#158][], [#159][] by [@tomcur][])

### Changed

* Breaking change: the deprecated conversion `From<Rgba8> for PremulColor<Srgb>` has been removed. Use `From<PremulRgba8> for PremulColor<Srgb>` instead. ([#157][] by [@tomcur][])
* Improve `no_std` support. ([#146][] by [@waywardmonkeys][])
* Make `{AlphaColor, OpaqueColor, PremulColor}::to_rgba8` faster. ([#166][] by [@tomcur][])

### Fixed

* Correctly determine analogous components between ACES2065-1 and other color spaces when converting,
  to carry missing components forward when interpolating colors with missing components in the ACES 2065-1 colorspace. ([#144][] by [@tomcur][])
* Fixed powerless hue component calculation for the HWB color space. ([#145][] by [@tomcur][])

## [0.2.3][] (2025-01-20)

This release has an [MSRV][] of 1.82.

### Added

* Support for the ACES2065-1 color space. ([#124][] by [@tomcur][])
* A documentation example implementing `ColorSpace`. ([#130][] by [@tomcur][])
* Conversions of `[u8; 4]` and packed `u32` into `Rgba8` and `PremulRgba8` are now provided. ([#135][] by [@tomcur][])
* Support construction of `AlphaColor<Srgb>`, `OpaqueColor<Srgb>` and `PremulColor<Srgb>` from rgb8 values. ([#136][] by [@waywardmonkeys][])

### Fixed

* Specify some `ColorSpace::WHITE_COMPONENTS` to higher precision. ([#128][], [#129][] by [@tomcur][])

## [0.2.2][] (2025-01-03)

This release has an [MSRV][] of 1.82.

### Fixed

* Colors in `XyzD65` are serialized as `xyz-d65` rather than `xyz`. ([#118][] by [@waywardmonkeys][])
* Alpha values are clamped at parse time. ([#119][] by [@waywardmonkeys][])

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
[@LaurenzV]: https://github.com/LaurenzV
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
[#118]: https://github.com/linebender/color/pull/118
[#119]: https://github.com/linebender/color/pull/119
[#124]: https://github.com/linebender/color/pull/124
[#128]: https://github.com/linebender/color/pull/128
[#129]: https://github.com/linebender/color/pull/129
[#130]: https://github.com/linebender/color/pull/130
[#135]: https://github.com/linebender/color/pull/135
[#136]: https://github.com/linebender/color/pull/136
[#139]: https://github.com/linebender/color/pull/139
[#144]: https://github.com/linebender/color/pull/144
[#145]: https://github.com/linebender/color/pull/145
[#146]: https://github.com/linebender/color/pull/146
[#149]: https://github.com/linebender/color/pull/149
[#153]: https://github.com/linebender/color/pull/153
[#155]: https://github.com/linebender/color/pull/155
[#156]: https://github.com/linebender/color/pull/156
[#157]: https://github.com/linebender/color/pull/157
[#158]: https://github.com/linebender/color/pull/158
[#159]: https://github.com/linebender/color/pull/159
[#164]: https://github.com/linebender/color/pull/164
[#165]: https://github.com/linebender/color/pull/165
[#166]: https://github.com/linebender/color/pull/166

[Unreleased]: https://github.com/linebender/color/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/linebender/color/releases/tag/v0.3.0
[0.2.3]: https://github.com/linebender/color/releases/tag/v0.2.3
[0.2.2]: https://github.com/linebender/color/releases/tag/v0.2.2
[0.2.1]: https://github.com/linebender/color/releases/tag/v0.2.1
[0.2.0]: https://github.com/linebender/color/releases/tag/v0.2.0
[0.1.0]: https://github.com/linebender/color/releases/tag/v0.1.0

[MSRV]: README.md#minimum-supported-rust-version-msrv
