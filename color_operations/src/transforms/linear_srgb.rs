// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Color transforms in sRGB-linear space.
//!
//! A set of color transformations described in [Filter Effects Module Level 1 § 13]
//!
//! These should all be applied to colors in the [`LinearSrgb`](color::LinearSrgb) color space, though some of them apply to any RGB color space.
//!
//! [Filter Effects Module Level 1 § 13](https://drafts.csswg.org/filter-effects/#ShorthandEquivalents)

// TODO - Give a longer description explaining which colorspace these apply to and why.

use crate::{ColorMatrix, ComponentTransfer};

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

// Relative luminance coefficients from WCAG 2.2, using the sRGB / Rec. 709 primaries.
// https://www.w3.org/TR/WCAG22/#dfn-relative-luminance
const LUMA_R: f32 = 0.2126;
const LUMA_G: f32 = 0.7152;
const LUMA_B: f32 = 0.0722;

// The Filter Effects hueRotate matrix is specified with older rounded luminance coefficients.
// Keep these literal values rather than substituting the higher-precision LUMA_* constants.
// https://www.w3.org/TR/filter-effects-1/#feColorMatrixElement
const HUE_ROTATE_LUMA_R: f32 = 0.213;
const HUE_ROTATE_LUMA_G: f32 = 0.715;
const HUE_ROTATE_LUMA_B: f32 = 0.072;

/// Create a matrix that linearly interpolates between the original and grayscale color.
///
/// An `amount` of `0.0` is the identity transform, and `1.0` maps the color components to
/// grayscale. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.1][grayscale].
///
/// [grayscale]: https://www.w3.org/TR/filter-effects-1/#grayscaleEquivalent
#[inline]
#[must_use]
pub const fn matrix_grayscale(amount: f32) -> ColorMatrix {
    matrix_saturate(1. - amount)
}

/// Create a matrix that linearly interpolates between the original and sepia color.
///
/// An `amount` of `0.0` is the identity transform, and `1.0` uses the full Filter Effects
/// `sepia(1)` matrix. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.2][sepia].
///
/// [sepia]: https://www.w3.org/TR/filter-effects-1/#sepiaEquivalent
#[inline]
#[must_use]
pub const fn matrix_sepia(amount: f32) -> ColorMatrix {
    let inverse = 1. - amount;
    ColorMatrix::new([
        [
            inverse + 0.393 * amount,
            0.769 * amount,
            0.189 * amount,
            0.,
            0.,
        ],
        [
            0.349 * amount,
            inverse + 0.686 * amount,
            0.168 * amount,
            0.,
            0.,
        ],
        [
            0.272 * amount,
            0.534 * amount,
            inverse + 0.131 * amount,
            0.,
            0.,
        ],
        [0., 0., 0., 1., 0.],
    ])
}

/// Create a matrix that adjusts saturation using relative luminance coefficients.
///
/// An `amount` of `1.0` is the identity transform, and `0.0` maps the color components to
/// grayscale. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.3][saturate].
///
/// [saturate]: https://www.w3.org/TR/filter-effects-1/#saturateEquivalent
#[inline]
#[must_use]
pub const fn matrix_saturate(amount: f32) -> ColorMatrix {
    ColorMatrix::new([
        [
            LUMA_R + amount * (1. - LUMA_R),
            LUMA_G - amount * LUMA_G,
            LUMA_B - amount * LUMA_B,
            0.,
            0.,
        ],
        [
            LUMA_R - amount * LUMA_R,
            LUMA_G + amount * (1. - LUMA_G),
            LUMA_B - amount * LUMA_B,
            0.,
            0.,
        ],
        [
            LUMA_R - amount * LUMA_R,
            LUMA_G - amount * LUMA_G,
            LUMA_B + amount * (1. - LUMA_B),
            0.,
            0.,
        ],
        [0., 0., 0., 1., 0.],
    ])
}

/// Create a matrix that rotates hue by `angle_degrees`.
///
/// This uses the SVG and Filter Effects `feColorMatrix` `hueRotate` matrix. The alpha
/// component is left unchanged. This assumes the first three components are RGB-like red,
/// green, and blue channels.
///
/// See [Filter Effects Module Level 1 § 13.1.4][hue-rotate].
///
/// [hue-rotate]: https://www.w3.org/TR/filter-effects-1/#huerotateEquivalent
#[inline]
#[must_use]
pub fn matrix_hue_rotate(angle_degrees: f32) -> ColorMatrix {
    let (sin, cos) = (angle_degrees * (core::f32::consts::PI / 180.)).sin_cos();

    ColorMatrix::new([
        [
            HUE_ROTATE_LUMA_R + cos * (1. - HUE_ROTATE_LUMA_R) - sin * HUE_ROTATE_LUMA_R,
            HUE_ROTATE_LUMA_G - cos * HUE_ROTATE_LUMA_G - sin * HUE_ROTATE_LUMA_G,
            HUE_ROTATE_LUMA_B - cos * HUE_ROTATE_LUMA_B + sin * (1. - HUE_ROTATE_LUMA_B),
            0.,
            0.,
        ],
        [
            HUE_ROTATE_LUMA_R - cos * HUE_ROTATE_LUMA_R + sin * 0.143,
            HUE_ROTATE_LUMA_G + cos * (1. - HUE_ROTATE_LUMA_G) + sin * 0.140,
            HUE_ROTATE_LUMA_B - cos * HUE_ROTATE_LUMA_B - sin * 0.283,
            0.,
            0.,
        ],
        [
            HUE_ROTATE_LUMA_R - cos * HUE_ROTATE_LUMA_R - sin * (1. - HUE_ROTATE_LUMA_R),
            HUE_ROTATE_LUMA_G - cos * HUE_ROTATE_LUMA_G + sin * HUE_ROTATE_LUMA_G,
            HUE_ROTATE_LUMA_B + cos * (1. - HUE_ROTATE_LUMA_B) + sin * HUE_ROTATE_LUMA_B,
            0.,
            0.,
        ],
        [0., 0., 0., 1., 0.],
    ])
}

/// Create a matrix that linearly interpolates between the original and inverted color.
///
/// An `amount` of `0.0` is the identity transform, and `1.0` maps each color component `c` to
/// `1.0 - c`. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.5][invert].
///
/// [invert]: https://www.w3.org/TR/filter-effects-1/#invertEquivalent
#[inline]
#[must_use]
pub const fn matrix_invert(amount: f32) -> ColorMatrix {
    let scale = 1. - 2. * amount;
    ColorMatrix::new([
        [scale, 0., 0., 0., amount],
        [0., scale, 0., 0., amount],
        [0., 0., scale, 0., amount],
        [0., 0., 0., 1., 0.],
    ])
}

/// Create a matrix that multiplies the alpha component by `amount`.
///
/// The color components are left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.6][opacity].
///
/// [opacity]: https://www.w3.org/TR/filter-effects-1/#opacityEquivalent
#[inline]
#[must_use]
pub const fn matrix_opacity(amount: f32) -> ColorMatrix {
    ColorMatrix::new([
        [1., 0., 0., 0., 0.],
        [0., 1., 0., 0., 0.],
        [0., 0., 1., 0., 0.],
        [0., 0., 0., amount, 0.],
    ])
}

/// Create a matrix that multiplies the color components by `amount`.
///
/// The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.7][brightness].
///
/// [brightness]: https://www.w3.org/TR/filter-effects-1/#brightnessEquivalent
#[inline]
#[must_use]
pub const fn matrix_brightness(amount: f32) -> ColorMatrix {
    ColorMatrix::new([
        [amount, 0., 0., 0., 0.],
        [0., amount, 0., 0., 0.],
        [0., 0., amount, 0., 0.],
        [0., 0., 0., 1., 0.],
    ])
}

/// Create a matrix that adjusts contrast around component value `0.5`.
///
/// An `amount` of `1.0` is the identity transform. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.8][contrast].
///
/// [contrast]: https://www.w3.org/TR/filter-effects-1/#contrastEquivalent
#[inline]
#[must_use]
pub const fn matrix_contrast(amount: f32) -> ColorMatrix {
    let offset = 0.5 * (1. - amount);
    ColorMatrix::new([
        [amount, 0., 0., 0., offset],
        [0., amount, 0., 0., offset],
        [0., 0., amount, 0., offset],
        [0., 0., 0., 1., 0.],
    ])
}

/// Create a matrix that moves relative luminance into alpha and clears color components.
///
/// This corresponds to the Filter Effects [`feColorMatrix`] `luminanceToAlpha` mode.
///
/// [`feColorMatrix`]: https://www.w3.org/TR/filter-effects-1/#feColorMatrixElement
#[inline]
#[must_use]
pub const fn matrix_luminance_to_alpha() -> ColorMatrix {
    ColorMatrix::new([
        [0., 0., 0., 0., 0.],
        [0., 0., 0., 0., 0.],
        [0., 0., 0., 0., 0.],
        [LUMA_R, LUMA_G, LUMA_B, 0., 0.],
    ])
}

/// Create a component transfer that linearly interpolates between the original and inverted
/// color.
///
/// An `amount` of `0.0` is the identity transform, and `1.0` maps each color component `c` to
/// `1.0 - c`. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.5][invert].
///
/// [invert]: https://www.w3.org/TR/filter-effects-1/#invertEquivalent
#[inline]
#[must_use]
pub const fn transfer_invert(amount: f32) -> ComponentTransfer<'static> {
    let scale = 1. - 2. * amount;
    ComponentTransfer::linear([scale, scale, scale, 1.], [amount, amount, amount, 0.])
}

/// Create a component transfer that multiplies the alpha component by `amount`.
///
/// The color components are left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.6][opacity].
///
/// [opacity]: https://www.w3.org/TR/filter-effects-1/#opacityEquivalent
#[inline]
#[must_use]
pub const fn transfer_opacity(amount: f32) -> ComponentTransfer<'static> {
    ComponentTransfer::linear([1., 1., 1., amount], [0.; 4])
}

/// Create a component transfer that multiplies the color components by `amount`.
///
/// The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.7][brightness].
///
/// [brightness]: https://www.w3.org/TR/filter-effects-1/#brightnessEquivalent
#[inline]
#[must_use]
pub const fn transfer_brightness(amount: f32) -> ComponentTransfer<'static> {
    ComponentTransfer::linear([amount, amount, amount, 1.], [0.; 4])
}

/// Create a component transfer that adjusts contrast around component value `0.5`.
///
/// An `amount` of `1.0` is the identity transform. The alpha component is left unchanged.
///
/// See [Filter Effects Module Level 1 § 13.1.8][contrast].
///
/// [contrast]: https://www.w3.org/TR/filter-effects-1/#contrastEquivalent
#[inline]
#[must_use]
pub const fn transfer_contrast(amount: f32) -> ComponentTransfer<'static> {
    let offset = 0.5 * (1. - amount);
    ComponentTransfer::linear([amount, amount, amount, 1.], [offset, offset, offset, 0.])
}

#[cfg(test)]
mod tests {
    use color::{AlphaColor, Srgb};

    use super::*;

    #[test]
    fn saturation_identity_is_identity() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            matrix_saturate(1.).apply(color).components,
            color.components
        );
    }

    #[test]
    fn grayscale_maps_to_luminance() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);
        let luma = 0.2 * 0.2126 + 0.4 * 0.7152 + 0.6 * 0.0722;

        assert_eq!(
            matrix_grayscale(1.).apply(color).components,
            [luma, luma, luma, 0.8]
        );
    }

    #[test]
    fn hue_rotate_zero_is_identity() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_approx_eq(
            matrix_hue_rotate(0.).apply(color).components,
            color.components,
        );
    }

    #[test]
    fn hue_rotate_uses_filter_effects_matrix() {
        let color = AlphaColor::<Srgb>::new([1., 0., 0., 1.]);

        assert_approx_eq(
            matrix_hue_rotate(90.).apply(color).components,
            [0., 0.356, -0.574, 1.],
        );
    }

    #[test]
    fn sepia_interpolates_to_full_sepia() {
        let color = AlphaColor::<Srgb>::new([1., 0., 0., 0.5]);

        assert_eq!(
            matrix_sepia(1.).apply(color).components,
            [0.393, 0.349, 0.272, 0.5]
        );
        assert_eq!(matrix_sepia(0.).apply(color).components, color.components);
    }

    #[test]
    fn luminance_to_alpha_clears_color_and_sets_alpha() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);
        let luma = 0.2 * 0.2126 + 0.4 * 0.7152 + 0.6 * 0.0722;

        assert_eq!(
            matrix_luminance_to_alpha().apply(color).components,
            [0., 0., 0., luma]
        );
    }

    #[test]
    fn brightness_scales_color_components() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            transfer_brightness(2.).apply(color).components,
            [0.4, 0.8, 1.2, 0.8]
        );
    }

    #[test]
    fn contrast_adjusts_around_midpoint() {
        let color = AlphaColor::<Srgb>::new([0.25, 0.5, 0.75, 0.8]);

        assert_eq!(
            transfer_contrast(2.).apply(color).components,
            [0., 0.5, 1., 0.8]
        );
    }

    #[test]
    fn invert_interpolates_to_inverted_color() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            transfer_invert(1.).apply(color).components,
            [0.8, 0.6, 0.39999998, 0.8]
        );
    }

    fn assert_approx_eq(actual: [f32; 4], expected: [f32; 4]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-6,
                "{actual:?} != {expected:?}"
            );
        }
    }
}
