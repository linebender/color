// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{ColorOperationTarget, TransferFunction};

/// Component-wise transfer over straight color components.
///
/// A `ComponentTransfer` applies an independent [`TransferFunction`] to each component in
/// `[c0, c1, c2, alpha]`.
///
/// This type has no color-space semantics. Convert colors into the intended working color space
/// before applying a transfer. [`ComponentTransfer::apply`] preserves whether the input color uses
/// straight or premultiplied alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentTransfer<'a> {
    /// Per-component transfer functions for `[c0, c1, c2, alpha]`.
    pub functions: [TransferFunction<'a>; 4],
}

impl<'a> ComponentTransfer<'a> {
    /// The identity component transfer.
    pub const IDENTITY: Self = Self::new([TransferFunction::Identity; 4]);

    /// Create a component transfer from per-component transfer functions.
    #[inline]
    #[must_use]
    pub const fn new(functions: [TransferFunction<'a>; 4]) -> Self {
        Self { functions }
    }

    /// Create a component transfer from per-component linear coefficients.
    #[inline]
    #[must_use]
    pub const fn linear(slopes: [f32; 4], intercepts: [f32; 4]) -> Self {
        Self::new([
            TransferFunction::linear(slopes[0], intercepts[0]),
            TransferFunction::linear(slopes[1], intercepts[1]),
            TransferFunction::linear(slopes[2], intercepts[2]),
            TransferFunction::linear(slopes[3], intercepts[3]),
        ])
    }

    /// Create a component transfer that multiplies the alpha component by `amount`.
    ///
    /// The color components are left unchanged.
    #[inline]
    #[must_use]
    pub const fn opacity(amount: f32) -> Self {
        Self::linear([1., 1., 1., amount], [0.; 4])
    }

    /// Create a component transfer that multiplies the color components by `amount`.
    ///
    /// The alpha component is left unchanged.
    #[inline]
    #[must_use]
    pub const fn brightness(amount: f32) -> Self {
        Self::linear([amount, amount, amount, 1.], [0.; 4])
    }

    /// Create a component transfer that adjusts contrast around component value `0.5`.
    ///
    /// An `amount` of `1.0` is the identity transform. The alpha component is left unchanged.
    #[inline]
    #[must_use]
    pub const fn contrast(amount: f32) -> Self {
        let offset = 0.5 * (1. - amount);
        Self::linear([amount, amount, amount, 1.], [offset, offset, offset, 0.])
    }

    /// Create a component transfer that linearly interpolates between the original and inverted
    /// color.
    ///
    /// An `amount` of `0.0` is the identity transform, and `1.0` maps each color component `c` to
    /// `1.0 - c`. The alpha component is left unchanged.
    #[inline]
    #[must_use]
    pub const fn invert(amount: f32) -> Self {
        let scale = 1. - 2. * amount;
        Self::linear([scale, scale, scale, 1.], [amount, amount, amount, 0.])
    }

    /// Apply this transfer to straight color components.
    #[inline]
    #[must_use]
    pub fn apply_components(self, components: [f32; 4]) -> [f32; 4] {
        [
            self.functions[0].apply(components[0]),
            self.functions[1].apply(components[1]),
            self.functions[2].apply(components[2]),
            self.functions[3].apply(components[3]),
        ]
    }

    /// Apply this transfer to premultiplied color components in a rectangular color space.
    ///
    /// This unpremultiplies the input, applies the transfer to straight components, and
    /// premultiplies the output color components by the output alpha. It does not clamp or clip
    /// the result.
    ///
    /// For typed [`color::PremulColor`] values, use [`ComponentTransfer::apply`] instead.
    #[inline]
    #[must_use]
    pub fn apply_premul_components(self, components: [f32; 4]) -> [f32; 4] {
        let [r, g, b, a] = components;
        let scale = if a == 0.0 { 1.0 } else { 1.0 / a };
        let [out_r, out_g, out_b, out_a] =
            self.apply_components([r * scale, g * scale, b * scale, a]);
        [out_r * out_a, out_g * out_a, out_b * out_a, out_a]
    }

    /// Apply this transfer to a typed color without converting its color space.
    ///
    /// This method preserves both the color-space marker and the alpha representation. To apply a
    /// transfer in another working color space, convert the color first, then convert the result
    /// back if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use color::{AlphaColor, Srgb};
    /// use color_operations::ComponentTransfer;
    ///
    /// let opacity = ComponentTransfer::opacity(0.5);
    /// let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 1.0]);
    ///
    /// assert_eq!(opacity.apply(color).components, [0.2, 0.4, 0.6, 0.5]);
    /// ```
    #[inline]
    #[must_use]
    pub fn apply<C: ColorOperationTarget>(self, color: C) -> C {
        color.apply_component_transfer(self)
    }
}

#[cfg(test)]
mod tests {
    use color::{AlphaColor, Srgb};

    use super::{ComponentTransfer, TransferFunction};

    #[test]
    fn identity_preserves_components() {
        let components = [0.2, 0.4, 0.6, 0.8];

        assert_eq!(
            ComponentTransfer::IDENTITY.apply_components(components),
            components
        );
    }

    #[test]
    fn linear_transfer_applies_per_component_coefficients() {
        let transfer = ComponentTransfer::linear([2., 3., 4., 0.5], [0.1, 0.2, 0.3, 0.4]);
        let components = [0.2, 0.3, 0.4, 0.5];

        assert_eq!(
            transfer.apply_components(components),
            [0.5, 1.1, 1.9000001, 0.65]
        );
    }

    #[test]
    fn table_interpolates_between_values() {
        let transfer = ComponentTransfer::new([
            TransferFunction::Table(&[0., 1.]),
            TransferFunction::Table(&[0., 0.5, 1.]),
            TransferFunction::Table(&[1.]),
            TransferFunction::Table(&[]),
        ]);

        assert_eq!(
            transfer.apply_components([0.25, 0.25, 0.25, 0.25]),
            [0.25, 0.25, 1., 0.25]
        );
    }

    #[test]
    fn table_uses_nearest_endpoint_outside_unit_interval() {
        let table = TransferFunction::Table(&[0.25, 0.75]);

        assert_eq!(table.apply(-0.5), 0.25);
        assert_eq!(table.apply(1.5), 0.75);
    }

    #[test]
    fn discrete_selects_steps() {
        let transfer = ComponentTransfer::new([
            TransferFunction::Discrete(&[0., 0.5, 1.]),
            TransferFunction::Discrete(&[1.]),
            TransferFunction::Discrete(&[]),
            TransferFunction::Identity,
        ]);

        assert_eq!(
            transfer.apply_components([0.1, 0.5, 0.25, 0.75]),
            [0., 1., 0.25, 0.75]
        );
        assert_eq!(transfer.functions[0].apply(0.5), 0.5);
        assert_eq!(transfer.functions[0].apply(1.0), 1.0);
    }

    #[test]
    fn opacity_scales_alpha() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            ComponentTransfer::opacity(0.5).apply(color).components,
            [0.2, 0.4, 0.6, 0.4]
        );
    }

    #[test]
    fn opacity_scales_premul_color() {
        let color = AlphaColor::<Srgb>::new([0.25, 0.5, 0.75, 0.5]).premultiply();

        assert_eq!(
            ComponentTransfer::opacity(0.5).apply(color).components,
            [0.0625, 0.125, 0.1875, 0.25]
        );
    }

    #[test]
    fn premul_components_use_straight_transfer_then_premultiply() {
        let transfer = ComponentTransfer::new([
            TransferFunction::Table(&[0., 1.]),
            TransferFunction::Table(&[0., 1.]),
            TransferFunction::Table(&[0., 1.]),
            TransferFunction::linear(0.5, 0.),
        ]);

        assert_eq!(
            transfer.apply_premul_components([0.25, 0.125, 0.0625, 0.5]),
            [0.125, 0.0625, 0.03125, 0.25]
        );
    }

    #[test]
    fn brightness_scales_color_components() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            ComponentTransfer::brightness(2.).apply(color).components,
            [0.4, 0.8, 1.2, 0.8]
        );
    }

    #[test]
    fn contrast_adjusts_around_midpoint() {
        let color = AlphaColor::<Srgb>::new([0.25, 0.5, 0.75, 0.8]);

        assert_eq!(
            ComponentTransfer::contrast(2.).apply(color).components,
            [0., 0.5, 1., 0.8]
        );
    }

    #[test]
    fn invert_interpolates_to_inverted_color() {
        let color = AlphaColor::<Srgb>::new([0.2, 0.4, 0.6, 0.8]);

        assert_eq!(
            ComponentTransfer::invert(1.).apply(color).components,
            [0.8, 0.6, 0.39999998, 0.8]
        );
    }
}
