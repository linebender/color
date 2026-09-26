// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::ColorOperationTarget;
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

/// An affine matrix over straight color components.
///
/// A `ColorMatrix` stores a row-major 4x5 matrix. It transforms a component vector
/// `[c0, c1, c2, alpha]` by multiplying the rows with `[c0, c1, c2, alpha, 1]`.
///
/// This type has no color-space semantics. Convert colors into the intended working color space
/// before applying a matrix. [`ColorMatrix::apply`] preserves whether the input color uses
/// straight or premultiplied alpha.
///
/// Constructors use arguments as provided. This crate does not apply CSS or SVG shorthand
/// clamping; callers implementing those specifications should clamp at the API boundary.
///
/// The named constructors that use RGB-specific formulas, such as [`ColorMatrix::grayscale`],
/// [`ColorMatrix::saturate`], [`ColorMatrix::hue_rotate`], [`ColorMatrix::sepia`], and
/// [`ColorMatrix::luminance_to_alpha`], assume the first three components are RGB-like red, green,
/// and blue channels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorMatrix {
    rows: [[f32; 5]; 4],
}

impl ColorMatrix {
    /// The identity color matrix.
    pub const IDENTITY: Self = Self::new([
        [1., 0., 0., 0., 0.],
        [0., 1., 0., 0., 0.],
        [0., 0., 1., 0., 0.],
        [0., 0., 0., 1., 0.],
    ]);

    /// Create a new color matrix from row-major 4x5 rows.
    #[inline]
    #[must_use]
    pub const fn new(rows: [[f32; 5]; 4]) -> Self {
        Self { rows }
    }

    /// Create a new color matrix from row-major 4x5 rows.
    #[inline]
    #[must_use]
    pub const fn from_rows(rows: [[f32; 5]; 4]) -> Self {
        Self::new(rows)
    }

    /// Return the row-major 4x5 rows of this matrix.
    #[inline]
    #[must_use]
    pub const fn rows(&self) -> &[[f32; 5]; 4] {
        &self.rows
    }

    /// Return the row-major 4x5 rows of this matrix by value.
    #[inline]
    #[must_use]
    pub const fn into_rows(self) -> [[f32; 5]; 4] {
        self.rows
    }

    /// Create a new color matrix from a flattened row-major 4x5 array.
    #[inline]
    #[must_use]
    pub const fn from_flattened(values: [f32; 20]) -> Self {
        Self::new([
            [values[0], values[1], values[2], values[3], values[4]],
            [values[5], values[6], values[7], values[8], values[9]],
            [values[10], values[11], values[12], values[13], values[14]],
            [values[15], values[16], values[17], values[18], values[19]],
        ])
    }

    /// Return this matrix as a flattened row-major 4x5 array.
    ///
    /// This is useful for APIs that consume SVG or Filter Effects style matrix values.
    ///
    /// # Examples
    ///
    /// ```
    /// use color_operations::ColorMatrix;
    ///
    /// let amount: f32 = 0.75;
    /// let flat = ColorMatrix::sepia(amount.clamp(0.0, 1.0)).to_flattened();
    ///
    /// assert_eq!(flat.len(), 20);
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_flattened(self) -> [f32; 20] {
        [
            self.rows[0][0],
            self.rows[0][1],
            self.rows[0][2],
            self.rows[0][3],
            self.rows[0][4],
            self.rows[1][0],
            self.rows[1][1],
            self.rows[1][2],
            self.rows[1][3],
            self.rows[1][4],
            self.rows[2][0],
            self.rows[2][1],
            self.rows[2][2],
            self.rows[2][3],
            self.rows[2][4],
            self.rows[3][0],
            self.rows[3][1],
            self.rows[3][2],
            self.rows[3][3],
            self.rows[3][4],
        ]
    }

    /// Return this matrix as a flattened row-major 4x5 slice.
    ///
    /// The returned slice always has length 20.
    #[inline]
    #[must_use]
    pub fn as_flattened(&self) -> &[f32] {
        self.rows.as_flattened()
    }

    /// Embed a 3x3 transform over the color components, preserving alpha.
    #[inline]
    pub const fn from_3x3(rows: [[f32; 3]; 3]) -> Self {
        Self::new([
            [rows[0][0], rows[0][1], rows[0][2], 0., 0.],
            [rows[1][0], rows[1][1], rows[1][2], 0., 0.],
            [rows[2][0], rows[2][1], rows[2][2], 0., 0.],
            [0., 0., 0., 1., 0.],
        ])
    }

    /// Create a matrix that multiplies the alpha component by `amount`.
    ///
    /// The color components are left unchanged.
    #[inline]
    #[must_use]
    pub const fn opacity(amount: f32) -> Self {
        Self::new([
            [1., 0., 0., 0., 0.],
            [0., 1., 0., 0., 0.],
            [0., 0., 1., 0., 0.],
            [0., 0., 0., amount, 0.],
        ])
    }

    /// Create a matrix that multiplies the color components by `amount`.
    ///
    /// The alpha component is left unchanged.
    #[inline]
    #[must_use]
    pub const fn brightness(amount: f32) -> Self {
        Self::new([
            [amount, 0., 0., 0., 0.],
            [0., amount, 0., 0., 0.],
            [0., 0., amount, 0., 0.],
            [0., 0., 0., 1., 0.],
        ])
    }

    /// Create a matrix that adjusts contrast around component value `0.5`.
    ///
    /// An `amount` of `1.0` is the identity transform. The alpha component is left unchanged.
    #[inline]
    #[must_use]
    pub const fn contrast(amount: f32) -> Self {
        let offset = 0.5 * (1. - amount);
        Self::new([
            [amount, 0., 0., 0., offset],
            [0., amount, 0., 0., offset],
            [0., 0., amount, 0., offset],
            [0., 0., 0., 1., 0.],
        ])
    }

    /// Create a matrix that linearly interpolates between the original and inverted color.
    ///
    /// An `amount` of `0.0` is the identity transform, and `1.0` maps each color component `c` to
    /// `1.0 - c`. The alpha component is left unchanged.
    #[inline]
    #[must_use]
    pub const fn invert(amount: f32) -> Self {
        let scale = 1. - 2. * amount;
        Self::new([
            [scale, 0., 0., 0., amount],
            [0., scale, 0., 0., amount],
            [0., 0., scale, 0., amount],
            [0., 0., 0., 1., 0.],
        ])
    }

    /// Create a matrix that adjusts saturation using relative luminance coefficients.
    ///
    /// An `amount` of `1.0` is the identity transform, and `0.0` maps the color components to
    /// grayscale. The alpha component is left unchanged.
    ///
    /// This assumes the first three components are RGB-like red, green, and blue channels.
    #[inline]
    #[must_use]
    pub const fn saturate(amount: f32) -> Self {
        Self::new([
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

    /// Create a matrix that linearly interpolates between the original and grayscale color.
    ///
    /// An `amount` of `0.0` is the identity transform, and `1.0` maps the color components to
    /// grayscale. The alpha component is left unchanged.
    ///
    /// This assumes the first three components are RGB-like red, green, and blue channels.
    #[inline]
    #[must_use]
    pub const fn grayscale(amount: f32) -> Self {
        Self::saturate(1. - amount)
    }

    /// Create a matrix that rotates hue by `angle_degrees`.
    ///
    /// This uses the SVG and Filter Effects `feColorMatrix` `hueRotate` matrix. The alpha
    /// component is left unchanged. This assumes the first three components are RGB-like red,
    /// green, and blue channels.
    #[inline]
    #[must_use]
    pub fn hue_rotate(angle_degrees: f32) -> Self {
        let (sin, cos) = (angle_degrees * (core::f32::consts::PI / 180.)).sin_cos();

        Self::new([
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

    /// Create a matrix that linearly interpolates between the original and sepia color.
    ///
    /// An `amount` of `0.0` is the identity transform, and `1.0` uses the full Filter Effects
    /// `sepia(1)` matrix. The alpha component is left unchanged.
    /// This assumes the first three components are RGB-like red, green, and blue channels.
    ///
    /// See [Filter Effects Module Level 1 § 15][sepia].
    ///
    /// [sepia]: https://www.w3.org/TR/filter-effects-1/#sepiaEquivalent
    #[inline]
    #[must_use]
    pub const fn sepia(amount: f32) -> Self {
        let inverse = 1. - amount;
        Self::new([
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

    /// Create a matrix that moves relative luminance into alpha and clears color components.
    ///
    /// This corresponds to the SVG and Filter Effects `feColorMatrix` `luminanceToAlpha` mode.
    /// It assumes the first three components are RGB-like red, green, and blue channels.
    #[inline]
    #[must_use]
    pub const fn luminance_to_alpha() -> Self {
        Self::new([
            [0., 0., 0., 0., 0.],
            [0., 0., 0., 0., 0.],
            [0., 0., 0., 0., 0.],
            [LUMA_R, LUMA_G, LUMA_B, 0., 0.],
        ])
    }

    /// Apply this matrix to straight color components.
    #[inline]
    #[must_use]
    pub const fn apply_components(self, components: [f32; 4]) -> [f32; 4] {
        let [c0, c1, c2, alpha] = components;
        [
            self.rows[0][0] * c0
                + self.rows[0][1] * c1
                + self.rows[0][2] * c2
                + self.rows[0][3] * alpha
                + self.rows[0][4],
            self.rows[1][0] * c0
                + self.rows[1][1] * c1
                + self.rows[1][2] * c2
                + self.rows[1][3] * alpha
                + self.rows[1][4],
            self.rows[2][0] * c0
                + self.rows[2][1] * c1
                + self.rows[2][2] * c2
                + self.rows[2][3] * alpha
                + self.rows[2][4],
            self.rows[3][0] * c0
                + self.rows[3][1] * c1
                + self.rows[3][2] * c2
                + self.rows[3][3] * alpha
                + self.rows[3][4],
        ]
    }

    /// Apply this matrix to premultiplied color components in a rectangular color space.
    ///
    /// This unpremultiplies the input, applies the matrix to straight components, and
    /// premultiplies the output color components by the output alpha. It does not clamp or clip
    /// the result.
    ///
    /// For typed [`color::PremulColor`] values, use [`ColorMatrix::apply`] instead.
    #[inline]
    #[must_use]
    pub const fn apply_premul_components(self, components: [f32; 4]) -> [f32; 4] {
        let [r, g, b, a] = components;
        let scale = if a == 0.0 { 1.0 } else { 1.0 / a };
        let [out_r, out_g, out_b, out_a] =
            self.apply_components([r * scale, g * scale, b * scale, a]);
        [out_r * out_a, out_g * out_a, out_b * out_a, out_a]
    }

    /// Apply this matrix directly to premultiplied color components in a rectangular color space.
    ///
    /// This method assumes [`ColorMatrix::is_premul_compatible`] is true. For matrices with that
    /// property, this produces the same result as [`ColorMatrix::apply_premul_components`] for
    /// valid premultiplied input in a rectangular color space without unpremultiplying the input.
    /// It does not clamp or clip the result.
    ///
    /// This method is intended for callers that can hoist the compatibility check outside a pixel
    /// loop.
    ///
    /// # Examples
    ///
    /// ```
    /// use color_operations::ColorMatrix;
    ///
    /// let matrix = ColorMatrix::grayscale(1.0);
    /// let premul = [0.2, 0.1, 0.0, 0.5];
    /// let out = if matrix.is_premul_compatible() {
    ///     matrix.apply_premul_compatible_components(premul)
    /// } else {
    ///     matrix.apply_premul_components(premul)
    /// };
    ///
    /// assert_eq!(out[3], 0.5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn apply_premul_compatible_components(self, components: [f32; 4]) -> [f32; 4] {
        let [r, g, b, a] = components;
        [
            self.rows[0][0] * r + self.rows[0][1] * g + self.rows[0][2] * b,
            self.rows[1][0] * r + self.rows[1][1] * g + self.rows[1][2] * b,
            self.rows[2][0] * r + self.rows[2][1] * g + self.rows[2][2] * b,
            a,
        ]
    }

    /// Returns whether this matrix can be applied directly to premultiplied color components.
    ///
    /// A compatible matrix preserves alpha and computes output color channels only from input color
    /// channels, without alpha terms or offsets. In a rectangular color space, applying the color
    /// rows to premultiplied components is then equivalent to unpremultiply, apply the matrix, and
    /// premultiply.
    #[inline]
    #[must_use]
    pub const fn is_premul_compatible(self) -> bool {
        self.rows[0][3] == 0.0
            && self.rows[0][4] == 0.0
            && self.rows[1][3] == 0.0
            && self.rows[1][4] == 0.0
            && self.rows[2][3] == 0.0
            && self.rows[2][4] == 0.0
            && self.rows[3][0] == 0.0
            && self.rows[3][1] == 0.0
            && self.rows[3][2] == 0.0
            && self.rows[3][3] == 1.0
            && self.rows[3][4] == 0.0
    }

    /// Apply this matrix to a typed color without converting its color space.
    ///
    /// This method preserves both the color-space marker and the alpha representation. To apply a
    /// matrix in another working color space, convert the color first, then convert the result
    /// back if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use color::{AlphaColor, Srgb};
    /// use color_operations::ColorMatrix;
    ///
    /// let opacity = ColorMatrix::opacity(0.5);
    /// let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 1.0]);
    ///
    /// assert_eq!(opacity.apply(color).components, [0.2, 0.4, 0.6, 0.5]);
    /// ```
    #[inline]
    #[must_use]
    pub fn apply<C: ColorOperationTarget>(self, color: C) -> C {
        color.apply_color_matrix(self)
    }

    /// Return a matrix that applies `self`, then applies `next`.
    ///
    /// For any component vector `c`, `self.then(next).apply_components(c)` is equivalent to
    /// `next.apply_components(self.apply_components(c))`.
    #[inline]
    #[must_use]
    pub const fn then(self, next: Self) -> Self {
        let first = self.rows;
        let next = next.rows;
        Self::new([
            [
                compose_component(&first, &next, 0, 0),
                compose_component(&first, &next, 0, 1),
                compose_component(&first, &next, 0, 2),
                compose_component(&first, &next, 0, 3),
                compose_component(&first, &next, 0, 4),
            ],
            [
                compose_component(&first, &next, 1, 0),
                compose_component(&first, &next, 1, 1),
                compose_component(&first, &next, 1, 2),
                compose_component(&first, &next, 1, 3),
                compose_component(&first, &next, 1, 4),
            ],
            [
                compose_component(&first, &next, 2, 0),
                compose_component(&first, &next, 2, 1),
                compose_component(&first, &next, 2, 2),
                compose_component(&first, &next, 2, 3),
                compose_component(&first, &next, 2, 4),
            ],
            [
                compose_component(&first, &next, 3, 0),
                compose_component(&first, &next, 3, 1),
                compose_component(&first, &next, 3, 2),
                compose_component(&first, &next, 3, 3),
                compose_component(&first, &next, 3, 4),
            ],
        ])
    }
}

const fn compose_component(
    first: &[[f32; 5]; 4],
    next: &[[f32; 5]; 4],
    row: usize,
    col: usize,
) -> f32 {
    next[row][0] * first[0][col]
        + next[row][1] * first[1][col]
        + next[row][2] * first[2][col]
        + next[row][3] * first[3][col]
        + if col == 4 { next[row][4] } else { 0. }
}

#[cfg(test)]
mod tests {
    use color::{AlphaColor, Srgb};

    use super::ColorMatrix;

    #[test]
    fn identity_preserves_components() {
        let components = [0.2, 0.4, 0.6, 0.8];

        assert_eq!(
            ColorMatrix::IDENTITY.apply_components(components),
            components
        );
    }

    #[test]
    fn flattened_matrix_uses_row_major_order() {
        let flat = [
            1., 2., 3., 4., 5., 6., 7., 8., 9., 10., 11., 12., 13., 14., 15., 16., 17., 18., 19.,
            20.,
        ];
        let rows = [
            [1., 2., 3., 4., 5.],
            [6., 7., 8., 9., 10.],
            [11., 12., 13., 14., 15.],
            [16., 17., 18., 19., 20.],
        ];

        let matrix = ColorMatrix::from_flattened(flat);

        assert_eq!(matrix.rows(), &rows);
        assert_eq!(matrix.as_flattened(), flat.as_slice());
        assert_eq!(matrix.to_flattened(), flat);
        assert_eq!(matrix.into_rows(), rows);
        assert_eq!(ColorMatrix::from_rows(rows).to_flattened(), flat);
    }

    #[test]
    fn from_3x3_preserves_alpha() {
        let matrix = ColorMatrix::from_3x3([[2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]);

        assert_eq!(
            matrix.apply_components([0.2, 0.3, 0.4, 0.5]),
            [0.4, 0.90000004, 1.6, 0.5]
        );
    }

    #[test]
    fn applies_to_alpha_color() {
        let matrix = ColorMatrix::new([
            [2., 0., 0., 0., 0.],
            [0., 3., 0., 0., 0.],
            [0., 0., 4., 0., 0.],
            [0., 0., 0., 0.5, 0.],
        ]);
        let color = AlphaColor::<Srgb>::new([0.2, 0.3, 0.4, 0.5]);

        assert_eq!(matrix.apply(color).components, [0.4, 0.90000004, 1.6, 0.25]);
    }

    #[test]
    fn opacity_scales_alpha() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            ColorMatrix::opacity(0.5).apply(color).components,
            [0.2, 0.4, 0.6, 0.4]
        );
    }

    #[test]
    fn composes_in_application_order() {
        let scale = ColorMatrix::from_3x3([[2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]);
        let offset = ColorMatrix::new([
            [1., 0., 0., 0., 0.1],
            [0., 1., 0., 0., 0.2],
            [0., 0., 1., 0., 0.3],
            [0., 0., 0., 1., 0.4],
        ]);
        let components = [0.2, 0.3, 0.4, 0.5];

        assert_eq!(
            scale.then(offset).apply_components(components),
            offset.apply_components(scale.apply_components(components))
        );
    }

    #[test]
    fn detects_premul_compatible_matrices() {
        assert!(ColorMatrix::IDENTITY.is_premul_compatible());
        assert!(ColorMatrix::grayscale(1.).is_premul_compatible());

        let alpha_offset = ColorMatrix::new([
            [1., 0., 0., 0., 0.],
            [0., 1., 0., 0., 0.],
            [0., 0., 1., 0., 0.],
            [0., 0., 0., 1., 0.1],
        ]);

        assert!(!alpha_offset.is_premul_compatible());
    }

    #[test]
    fn applies_to_premul_color() {
        let matrix = ColorMatrix::new([
            [1., 0., 0., 0., 0.25],
            [0., 1., 0., 0., 0.],
            [0., 0., 1., 0., 0.],
            [0., 0., 0., 1., 0.],
        ]);
        let color = AlphaColor::<Srgb>::new([0.25, 0.5, 0.75, 0.5]).premultiply();

        assert_eq!(matrix.apply(color).components, [0.25, 0.25, 0.375, 0.5]);
    }

    #[test]
    fn premul_compatible_path_matches_straight_path() {
        let matrix = ColorMatrix::from_3x3([[2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]);
        let components = [0.25, 0.125, 0.0625, 0.5];

        assert_eq!(
            matrix.apply_premul_compatible_components(components),
            matrix.apply_premul_components(components)
        );
    }

    #[test]
    fn premul_compatible_path_does_not_clip_color_to_alpha() {
        let matrix = ColorMatrix::from_3x3([[2., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);

        assert_eq!(
            matrix.apply_premul_compatible_components([0.5, 0., 0., 0.5]),
            [1., 0., 0., 0.5]
        );
    }

    #[test]
    fn premul_general_path_can_create_color_from_transparent_black() {
        let matrix = ColorMatrix::new([
            [0., 0., 0., 0., 0.5],
            [0., 0., 0., 0., 0.25],
            [0., 0., 0., 0., 0.0],
            [0., 0., 0., 0., 0.5],
        ]);

        assert_eq!(
            matrix.apply_premul_components([0., 0., 0., 0.]),
            [0.25, 0.125, 0., 0.5]
        );
    }

    #[test]
    fn saturation_identity_is_identity() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            ColorMatrix::saturate(1.).apply(color).components,
            color.components
        );
    }

    #[test]
    fn grayscale_maps_to_luminance() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);
        let luma = 0.2 * 0.2126 + 0.4 * 0.7152 + 0.6 * 0.0722;

        assert_eq!(
            ColorMatrix::grayscale(1.).apply(color).components,
            [luma, luma, luma, 0.8]
        );
    }

    #[test]
    fn hue_rotate_zero_is_identity() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_approx_eq(
            ColorMatrix::hue_rotate(0.).apply(color).components,
            color.components,
        );
    }

    #[test]
    fn hue_rotate_uses_filter_effects_matrix() {
        let color = AlphaColor::<Srgb>::new([1., 0., 0., 1.]);

        assert_approx_eq(
            ColorMatrix::hue_rotate(90.).apply(color).components,
            [0., 0.356, -0.574, 1.],
        );
    }

    #[test]
    fn sepia_interpolates_to_full_sepia() {
        let color = AlphaColor::<Srgb>::new([1., 0., 0., 0.5]);

        assert_eq!(
            ColorMatrix::sepia(1.).apply(color).components,
            [0.393, 0.349, 0.272, 0.5]
        );
        assert_eq!(
            ColorMatrix::sepia(0.).apply(color).components,
            color.components
        );
    }

    #[test]
    fn luminance_to_alpha_clears_color_and_sets_alpha() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);
        let luma = 0.2 * 0.2126 + 0.4 * 0.7152 + 0.6 * 0.0722;

        assert_eq!(
            ColorMatrix::luminance_to_alpha().apply(color).components,
            [0., 0., 0., luma]
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
