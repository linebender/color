// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Color is a Rust crate which implements color space conversions, targeting at least
//! [CSS Color Level 4].
//!
//! ## Main types
//!
//! The crate has two approaches to representing color in the Rust type system: a set of
//! types with static color space as part of the types, and [`DynamicColor`]
//! in which the color space is represented at runtime.
//!
//! The static color types come in three variants: [`OpaqueColor`] without an
//! alpha channel, [`AlphaColor`] with a separate alpha channel, and [`PremulColor`] with
//! premultiplied alpha. The last type is particularly useful for making interpolation and
//! compositing more efficient. These have a marker type parameter, indicating which
//! [`ColorSpace`] they are in. Conversion to another color space uses the `convert` method
//! on each of these types. The static types are open-ended, as it's possible to implement
//! this trait for new color spaces.
//!
//! ## Scope and goals
//!
//! Color in its entirety is an extremely deep and complex topic. It is completely impractical
//! for a single crate to meet all color needs. The goal of this one is to strike a balance,
//! providing color capabilities while also keeping things simple and efficient.
//!
//! The main purpose of this crate is to provide a good set of types for representing colors,
//! along with conversions between them and basic manipulations, especially interpolation. A
//! major inspiration is the [CSS Color Level 4] draft spec; we implement most of the operations
//! and strive for correctness.
//!
//! A primary use case is rendering, including color conversions and methods for preparing
//! gradients. The crate should also be suitable for document authoring and editing, as it
//! contains methods for parsing and serializing colors with CSS Color 4 compatible syntax.
//!
//! Simplifications include:
//!   * Always using `f32` to represent component values.
//!   * Only handling 3-component color spaces (plus optional alpha).
//!   * Choosing a fixed, curated set of color spaces for dynamic color types.
//!   * Choosing linear sRGB as the central color space.
//!   * Keeping white point implicit.
//!
//! A number of other tasks are out of scope for this crate:
//!   * Print color spaces (CMYK).
//!   * Spectral colors.
//!   * Color spaces with more than 3 components generally.
//!   * [ICC] color profiles.
//!   * [ACES] color transforms.
//!   * Appearance models and other color science not needed for rendering.
//!   * Quantizing and packing to lower bit depths.
//!
//! The [`Rgba8`] and [`PremulRgba8`] types are a partial exception to this last item, as
//! those representation are ubiquitous and requires special logic for serializing to
//! maximize compatibility.
//!
//! Some of these capabilities may be added as other crates within the `color` repository,
//! and we will also facilitate interoperability with other color crates in the Rust
//! ecosystem as needed.
//!
//! ## Features
//!
//! - `std` (enabled by default): Get floating point functions from the standard library
//!   (likely using your target's libc).
//! - `libm`: Use floating point implementations from [libm][].
//! - `bytemuck`: Implement traits from `bytemuck` on [`AlphaColor`], [`ColorSpaceTag`],
//!   [`HueDirection`], [`OpaqueColor`], [`PremulColor`], [`PremulRgba8`], and [`Rgba8`].
//! - `serde`: Implement `serde::Deserialize` and `serde::Serialize` on [`AlphaColor`],
//!   [`DynamicColor`], [`OpaqueColor`], [`PremulColor`], [`PremulRgba8`], and [`Rgba8`].
//!
//! At least one of `std` and `libm` is required; `std` overrides `libm`.
//!
//! [CSS Color Level 4]: https://www.w3.org/TR/css-color-4/
//! [ICC]: https://color.org/
//! [ACES]: https://acescentral.com/
#![cfg_attr(feature = "libm", doc = "[libm]: libm")]
#![cfg_attr(not(feature = "libm"), doc = "[libm]: https://crates.io/crates/libm")]
// LINEBENDER LINT SET - lib.rs - v1
// See https://linebender.org/wiki/canonical-lints/
// These lints aren't included in Cargo.toml because they
// shouldn't apply to examples and tests
#![warn(unused_crate_dependencies)]
#![warn(clippy::print_stdout, clippy::print_stderr)]
// END LINEBENDER LINT SET
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

pub mod cache_key;
mod color;
mod colorspace;
mod dynamic;
mod flags;
mod gradient;
pub mod palette;
mod rgba8;
mod serialize;
mod tag;
mod x11_colors;

// Note: this may become feature-gated; we'll decide this soon
// (This line is isolated so that the comment binds to it with import ordering)
mod parse;

#[cfg(feature = "bytemuck")]
mod impl_bytemuck;

#[cfg(all(not(feature = "std"), not(test)))]
mod floatfuncs;

pub use color::{AlphaColor, HueDirection, OpaqueColor, PremulColor};
pub use colorspace::{
    A98Rgb, Aces2065_1, AcesCg, ColorSpace, ColorSpaceLayout, DisplayP3, Hsl, Hwb, Lab, Lch,
    LinearSrgb, Oklab, Oklch, ProphotoRgb, Rec2020, Srgb, XyzD50, XyzD65,
};
pub use dynamic::{DynamicColor, Interpolator};
pub use flags::{Flags, Missing};
pub use gradient::{gradient, GradientIter};
pub use parse::{parse_color, parse_color_prefix, ParseError};
pub use rgba8::{PremulRgba8, Rgba8};
pub use tag::ColorSpaceTag;

const fn u8_to_f32(x: u8) -> f32 {
    x as f32 * (1.0 / 255.0)
}

fn matmul(m: &[[f32; 3]; 3], x: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * x[0] + m[0][1] * x[1] + m[0][2] * x[2],
        m[1][0] * x[0] + m[1][1] * x[1] + m[1][2] * x[2],
        m[2][0] * x[0] + m[2][1] * x[1] + m[2][2] * x[2],
    ]
}

impl AlphaColor<Srgb> {
    /// Create a color from 8-bit rgba values.
    ///
    /// Note: for conversion from the [`Rgba8`] type, just use the `From` trait.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let components = [u8_to_f32(r), u8_to_f32(g), u8_to_f32(b), u8_to_f32(a)];
        Self::new(components)
    }
}

impl PremulColor<Srgb> {
    /// Create a color from pre-multiplied 8-bit rgba values.
    ///
    /// Note: for conversion from the [`PremulRgba8`] type, just use the `From` trait.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let components = [u8_to_f32(r), u8_to_f32(g), u8_to_f32(b), u8_to_f32(a)];
        Self::new(components)
    }
}

// Keep clippy from complaining about unused libm in nostd test case.
#[cfg(feature = "libm")]
#[expect(unused, reason = "keep clippy happy")]
fn ensure_libm_dependency_used() -> f32 {
    libm::sqrtf(4_f32)
}
