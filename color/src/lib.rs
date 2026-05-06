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
//!   * Keeping white point implicit in the general conversion operations.
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
// LINEBENDER LINT SET - lib.rs - v4
// See https://linebender.org/wiki/canonical-lints/
// These lints shouldn't apply to examples or tests.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
// These lints shouldn't apply to examples.
#![warn(clippy::print_stdout, clippy::print_stderr)]
// Targeting e.g. 32-bit means structs containing usize can give false positives for 64-bit.
#![cfg_attr(target_pointer_width = "64", warn(clippy::trivially_copy_pass_by_ref))]
// END LINEBENDER LINT SET
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

pub mod cache_key;
mod chromaticity;
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

use crate::parse::{color_from_4bit_hex, get_4bit_hex_channels};
pub use chromaticity::Chromaticity;
pub use color::{AlphaColor, HueDirection, OpaqueColor, PremulColor};
pub use colorspace::{
    A98Rgb, Aces2065_1, AcesCg, ColorSpace, ColorSpaceLayout, DisplayP3, Hsl, Hwb, Lab, Lch,
    LinearSrgb, Oklab, Oklch, ProphotoRgb, Rec2020, Srgb, XyzD50, XyzD65,
};
pub use dynamic::{DynamicColor, Interpolator, UnpremultipliedInterpolator};
pub use flags::{Flags, Missing};
pub use gradient::{GradientIter, UnpremultipliedGradientIter, gradient, gradient_unpremultiplied};
pub use parse::{ParseError, parse_color, parse_color_prefix};
pub use rgba8::{PremulRgba8, Rgba8};
pub use tag::ColorSpaceTag;

const fn u8_to_f32(x: u8) -> f32 {
    x as f32 * (1.0 / 255.0)
}

/// Multiplication `m * x` of a 3x3-matrix `m` and a 3-vector `x`.
const fn matvecmul(m: &[[f32; 3]; 3], x: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * x[0] + m[0][1] * x[1] + m[0][2] * x[2],
        m[1][0] * x[0] + m[1][1] * x[1] + m[1][2] * x[2],
        m[2][0] * x[0] + m[2][1] * x[1] + m[2][2] * x[2],
    ]
}

/// Multiplication `ma * mb` of two 3x3-matrices `ma` and `mb`.
const fn matmatmul(ma: &[[f32; 3]; 3], mb: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [
            ma[0][0] * mb[0][0] + ma[0][1] * mb[1][0] + ma[0][2] * mb[2][0],
            ma[0][0] * mb[0][1] + ma[0][1] * mb[1][1] + ma[0][2] * mb[2][1],
            ma[0][0] * mb[0][2] + ma[0][1] * mb[1][2] + ma[0][2] * mb[2][2],
        ],
        [
            ma[1][0] * mb[0][0] + ma[1][1] * mb[1][0] + ma[1][2] * mb[2][0],
            ma[1][0] * mb[0][1] + ma[1][1] * mb[1][1] + ma[1][2] * mb[2][1],
            ma[1][0] * mb[0][2] + ma[1][1] * mb[1][2] + ma[1][2] * mb[2][2],
        ],
        [
            ma[2][0] * mb[0][0] + ma[2][1] * mb[1][0] + ma[2][2] * mb[2][0],
            ma[2][0] * mb[0][1] + ma[2][1] * mb[1][1] + ma[2][2] * mb[2][1],
            ma[2][0] * mb[0][2] + ma[2][1] * mb[1][2] + ma[2][2] * mb[2][2],
        ],
    ]
}

/// Multiplication `ma * mb` of a 3x3-matrix `ma` by a 3x3-diagonal matrix `mb`.
///
/// Diagonal matrix `mb` is given by
///
/// ```text
/// [ mb[0] 0     0     ]
/// [ 0     mb[1] 0     ]
/// [ 0     0     mb[2] ]
/// ```
const fn matdiagmatmul(ma: &[[f32; 3]; 3], mb: [f32; 3]) -> [[f32; 3]; 3] {
    [
        [ma[0][0] * mb[0], ma[0][1] * mb[1], ma[0][2] * mb[2]],
        [ma[1][0] * mb[0], ma[1][1] * mb[1], ma[1][2] * mb[2]],
        [ma[2][0] * mb[0], ma[2][1] * mb[1], ma[2][2] * mb[2]],
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

    /// Create a color from 8-bit rgb values with an opaque alpha.
    ///
    /// Note: for conversion from the [`Rgba8`] type, just use the `From` trait.
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        let components = [u8_to_f32(r), u8_to_f32(g), u8_to_f32(b), 1.];
        Self::new(components)
    }

    /// Create an sRGB color from a hexadecimal string, such as `"#8a2be2"` (<span style="background-color:#8a2be2;padding:0 0.7em;border:1px solid"></span>).
    ///
    /// Certain code editors may provide a color picker for input strings of this format, making this method
    /// preferable to [`from_rgb8`](`Self::from_rgb8`) or [`from_rgba8`](`Self::from_rgba8`) for colors which may need to be experimented with.
    ///
    /// The leading `#` in the input is optional, but it is recommended to include it.
    /// The input is provided in RGBA order, and valid inputs are of the form `#RGB`, `#RGBA`, `#RRGGBB` or `#RRGGBBAA`.
    /// `A-F` in the input string may be upper or lowercase.
    ///
    /// This function is designed for use in const contexts; for user-provided values, you can use
    /// [`parse_color`], which covers a wider variety of input forms in CSS syntax, or
    /// [`try_from_hex`](Self::try_from_hex) which returns an error instead of panicking.
    ///
    /// # Example
    ///
    /// ```
    /// # use color::{AlphaColor, Srgb};
    /// const BUTTON_COLOR: AlphaColor<Srgb> = AlphaColor::from_hex("#8a2be2");
    /// ```
    ///
    /// # Panics
    ///
    /// If the input string contains anything other than an optional `#` and 3, 4, 6, or 8 hexadecimal digits.
    pub const fn from_hex(hex: &str) -> Self {
        match Self::try_from_hex(hex) {
            Ok(color) => color,
            Err(ParseError::WrongNumberOfHexDigits) => {
                panic!("An invalid number of hexadecimal digits was provided.");
            }
            Err(ParseError::ExpectedEndOfString) => {
                panic!("Input to from_hex contains characters after hexadecimal digits.");
            }
            Err(_) => {
                unreachable!()
            }
        }
    }

    /// Create a color from a hexadecimal string, such as `"#8a2be2"` (<span style="background-color:#8a2be2;padding:0 0.7em;border:1px solid"></span>).
    ///
    /// Same as [`from_hex`](Self::from_hex), but returns an error in cases where that method panics.
    pub const fn try_from_hex(mut hex: &str) -> Result<Self, ParseError> {
        // Strip an optional '#' from the start. We can't use `strip_prefix` as it isn't const.
        if !hex.is_empty() && hex.as_bytes()[0] == b'#' {
            hex = hex.split_at(1).1;
        }

        let bit_hex = get_4bit_hex_channels(hex);
        match bit_hex {
            Ok((count, channels)) => {
                if count != hex.len() {
                    return Err(ParseError::ExpectedEndOfString);
                }
                Ok(color_from_4bit_hex(channels))
            }
            Err(e) => Err(e),
        }
    }
}

impl OpaqueColor<Srgb> {
    /// Create a color from 8-bit rgb values.
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        let components = [u8_to_f32(r), u8_to_f32(g), u8_to_f32(b)];
        Self::new(components)
    }

    /// Create an sRGB color from a hexadecimal string, such as `"#8a2be2"` (<span style="background-color:#8a2be2;padding:0 0.7em;border:1px solid"></span>).
    ///
    /// Certain code editors may provide a color picker for input strings of this format, making this method
    /// preferable to [`from_rgb8`](`Self::from_rgb8`) for colors which may need to be experimented with.
    ///
    /// The leading `#` in the input is optional, but it is recommended to include it.
    /// The input is provided in RGBA order, and valid inputs are of the form `#RGB`, `#RRGGBB` or `#RRGGBB`.
    /// `A-F` in the input string may be upper or lowercase.
    ///
    /// This function is designed for use in const contexts; for user-provided values, you can use
    /// [`parse_color`], which covers a wider variety of input forms in CSS syntax, or
    /// [`try_from_hex`](Self::try_from_hex) which returns an error instead of panicking.
    ///
    /// # Example
    ///
    /// ```
    /// # use color::{OpaqueColor, Srgb};
    /// const BUTTON_COLOR: OpaqueColor<Srgb> = OpaqueColor::from_hex("#8a2be2");
    /// ```
    ///
    /// # Panics
    ///
    /// If the input string contains anything other than an optional `#` and 3, or 6 hexadecimal digits.
    pub const fn from_hex(hex: &str) -> Self {
        match Self::try_from_hex(hex) {
            Ok(color) => color,
            Err(ParseError::WrongNumberOfHexDigits) => {
                panic!("An invalid number of hexadecimal digits was provided.");
            }
            Err(ParseError::ExpectedEndOfString) => {
                panic!("Input to from_hex contains characters after hexadecimal digits.");
            }
            Err(_) => {
                unreachable!()
            }
        }
    }

    /// Create a color from a hexadecimal string, such as `"#8a2be2"` (<span style="background-color:#8a2be2;padding:0 0.7em;border:1px solid"></span>).
    ///
    /// Same as [`from_hex`](Self::from_hex), but returns an error in cases where that method panics.
    pub const fn try_from_hex(mut hex: &str) -> Result<Self, ParseError> {
        // Strip an optional '#' from the start. We can't use `strip_prefix` as it isn't const.
        if !hex.is_empty() && hex.as_bytes()[0] == b'#' {
            hex = hex.split_at(1).1;
        }

        let bit_hex = get_4bit_hex_channels(hex);
        match bit_hex {
            Ok((count, channels)) => {
                if count != hex.len() {
                    return Err(ParseError::ExpectedEndOfString);
                }
                if count != 3 && count != 6 {
                    return Err(ParseError::WrongNumberOfHexDigits);
                }
                Ok(color_from_4bit_hex(channels).discard_alpha())
            }
            Err(e) => Err(e),
        }
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

    /// Create a color from 8-bit rgb values with an opaque alpha.
    ///
    /// Note: for conversion from the [`Rgba8`] type, just use the `From` trait.
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        let components = [u8_to_f32(r), u8_to_f32(g), u8_to_f32(b), 1.];
        Self::new(components)
    }
}

// Keep clippy from complaining about unused libm in nostd test case.
#[cfg(feature = "libm")]
#[expect(unused, reason = "keep clippy happy")]
fn ensure_libm_dependency_used() -> f32 {
    libm::sqrtf(4_f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALPHA_FROM_HEX_IS_CONST: AlphaColor<Srgb> = AlphaColor::from_hex("#8a2be2");
    #[test]
    fn alpha_from_hex() {
        let color = AlphaColor::from_hex("#8a2be2");
        assert_eq!(
            color.to_rgba8(),
            Rgba8::from_u8_array([0x8a, 0x2b, 0xe2, 0xff])
        );
        assert_eq!(color, ALPHA_FROM_HEX_IS_CONST);

        let with = AlphaColor::from_hex("#aabbcc");
        let without = AlphaColor::from_hex("aabbcc");
        assert_eq!(with, without);

        let short = AlphaColor::from_hex("#abc");
        let long = AlphaColor::from_hex("#aabbcc");
        assert_eq!(short, long);

        let short_alpha = AlphaColor::from_hex("#abcd");
        let long_alpha = AlphaColor::from_hex("#aabbccdd");
        assert_eq!(short_alpha, long_alpha);

        let lower = AlphaColor::from_hex("#8a2be28f");
        let upper = AlphaColor::from_hex("#8A2BE28F");
        assert_eq!(lower, upper);
    }

    #[test]
    fn alpha_try_from_hex_errors() {
        // 'g' is not a valid hex digit
        assert!(AlphaColor::<Srgb>::try_from_hex("#gg0000").is_err());
        // 5 digit color isn't defined.
        assert!(AlphaColor::<Srgb>::try_from_hex("#12345").is_err());
        assert!(AlphaColor::<Srgb>::try_from_hex("").is_err());
    }

    const OPAQUE_FROM_HEX_IS_CONST: OpaqueColor<Srgb> = OpaqueColor::from_hex("#8a2be2");
    #[test]
    fn opaque_from_hex() {
        let color = OpaqueColor::from_hex("#8a2be2");
        assert_eq!(
            color.to_rgba8(),
            Rgba8::from_u8_array([0x8a, 0x2b, 0xe2, 0xff])
        );
        assert_eq!(color, OPAQUE_FROM_HEX_IS_CONST);

        let with = OpaqueColor::from_hex("#aabbcc");
        let without = OpaqueColor::from_hex("aabbcc");
        assert_eq!(with, without);

        let short = OpaqueColor::from_hex("#abc");
        let long = OpaqueColor::from_hex("#aabbcc");
        assert_eq!(short, long);

        let short_alpha = OpaqueColor::from_hex("#abcd");
        let long_alpha = OpaqueColor::from_hex("#aabbccdd");
        assert_eq!(short_alpha, long_alpha);

        let lower = OpaqueColor::from_hex("#8a2be28f");
        let upper = OpaqueColor::from_hex("#8A2BE28F");
        assert_eq!(lower, upper);
    }
    #[test]
    fn opaque_try_from_hex_errors() {
        // 'g' is not a valid hex digit
        assert!(OpaqueColor::try_from_hex("#gg0000").is_err());
        // 5 digit color isn't defined.
        assert!(OpaqueColor::try_from_hex("#12345").is_err());
        // 4 digit color isn't allowed for an opaque color.
        assert!(OpaqueColor::try_from_hex("#123f").is_err());
        // 8 digit color isn't allowed for an opaque color.
        assert!(OpaqueColor::try_from_hex("#12233480").is_err());
        assert!(OpaqueColor::try_from_hex("").is_err());
    }
}
