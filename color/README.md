<div align="center">

# Color

**TODO: Add tagline**

[![Linebender Zulip, #color channel](https://img.shields.io/badge/Linebender-%23color-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/channel/466849-color)
[![dependency status](https://deps.rs/repo/github/linebender/color/status.svg)](https://deps.rs/repo/github/linebender/color)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![Build status](https://github.com/linebender/color/workflows/CI/badge.svg)](https://github.com/linebender/color/actions)
[![Crates.io](https://img.shields.io/crates/v/color.svg)](https://crates.io/crates/color)
[![Docs](https://docs.rs/color/badge.svg)](https://docs.rs/color)

</div>

<!-- We use cargo-rdme to update the README with the contents of lib.rs.
To edit the following section, update it in lib.rs, then run:
cargo rdme --workspace-project=color --heading-base-level=0
Full documentation at https://github.com/orium/cargo-rdme -->

<!-- Intra-doc links used in lib.rs should be evaluated here. 
See https://linebender.org/blog/doc-include/ for related discussion. -->
[libm]: https://crates.io/crates/libm

<!-- cargo-rdme start -->

Color is a Rust crate which implements color space conversions, targeting at least CSS4 color.

## Scope and goals

Color in its entirety is an extremely deep and complex topic. It is completely impractical
for a single crate to meet all color needs. The goal of this one is to strike a balance,
providing color capabilities while also keeping things simple and efficient.

The main purpose of this crate is to provide a good set of types for representing colors,
along with conversions between them and basic manipulations, especially interpolation. A
major inspiration is the CSS Color Level 4 draft spec; we implement most of the operations
and strive for correctness.

Simplifications include:
  * Always using `f32` to represent component values.
  * Only handling 3-component color spaces (plus optional alpha).
  * Choosing a fixed, curated set of color spaces for dynamic color types.
  * Choosing linear sRGB as the central color space.
  * Keeping white point implicit.

A number of other tasks are out of scope for this crate:
  * Print color spaces (CMYK).
  * Spectral colors.
  * Color spaces with more than 3 components generally.
  * [ICC] color profiles.
  * [ACES] color transforms.
  * Appearance models and other color science not needed for rendering.
  * Quantizing and packing to lower bit depths.

## Main types

The crate has two approaches to representing color in the Rust type system: a set of
types with static color space as part of the types, and [`DynamicColor`] in which the
color space is represented at runtime.

The static color types come in three variants: [`OpaqueColor`] without an alpha channel,
[`AlphaColor`] with a separate alpha channel, and [`PremulColor`] with premultiplied
alpha. The last type is particularly useful for making interpolation and compositing more
efficient. All have a generic type parameter with a trait bound of [`ColorSpace`], a
zero sized type. The static types are open-ended, as it's possible to implement this
trait for new color spaces.

## Features

- `std` (enabled by default): Get floating point functions from the standard library (likely using your target's libc).
- `libm`: Use floating point implementations from [libm][].
- `bytemuck`: Implement traits from `bytemuck` on [`AlphaColor`], [`OpaqueColor`], [`PremulColor`], and [`Rgba8`].

At least one of `std` and `libm` is required; `std` overrides `libm`.

[ICC]: https://color.org/
[ACES]: https://acescentral.com/

<!-- cargo-rdme end -->

## Minimum supported Rust Version (MSRV)

This version of Color has been verified to compile with **Rust 1.82** and later.

Future versions of Color might increase the Rust version requirement.
It will not be treated as a breaking change and as such can even happen with small patch releases.

<details>
<summary>Click here if compiling fails.</summary>

As time has passed, some of Color's dependencies could have released versions with a higher Rust requirement.
If you encounter a compilation issue due to a dependency and don't want to upgrade your Rust toolchain, then you could downgrade the dependency.

```sh
# Use the problematic dependency's name and version
cargo update -p package_name --precise 0.1.1
```

</details>

## Community

[![Linebender Zulip](https://img.shields.io/badge/Xi%20Zulip-%23color-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/channel/466849-color)

Discussion of Color development happens in the [Linebender Zulip](https://xi.zulipchat.com/), specifically the [#color channel](https://xi.zulipchat.com/#narrow/channel/466849-color).
All public content can be read without logging in.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Contributions are welcome by pull request. The [Rust code of conduct] applies.
Please feel free to add your name to the [AUTHORS] file in any substantive pull request.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Rust Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct
[AUTHORS]: ./AUTHORS
