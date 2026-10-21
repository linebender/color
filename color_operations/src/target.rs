// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{ColorMatrix, ComponentTransfer};

/// A typed color that can be transformed by color operations.
///
/// This trait is implemented for [`color::AlphaColor`], [`color::PremulColor`], and
/// [`color::DynamicColor`]. It is sealed so that `color_operations` can preserve the
/// alpha-representation invariants for every supported target type.
pub trait ColorOperationTarget: private::Sealed + Sized {
    /// Apply a component transfer to this color.
    ///
    /// The result preserves the input color-space identity and alpha representation. For
    /// [`color::DynamicColor`], missing-component flags are preserved and named-color state is
    /// discarded.
    #[must_use]
    fn apply_component_transfer(self, transfer: ComponentTransfer<'_>) -> Self;

    /// Apply a color matrix to this color.
    ///
    /// The result preserves the input color-space identity and alpha representation. For
    /// [`color::DynamicColor`], missing-component flags are preserved and named-color state is
    /// discarded.
    #[must_use]
    fn apply_color_matrix(self, matrix: ColorMatrix) -> Self;
}

impl<CS: color::ColorSpace> ColorOperationTarget for color::AlphaColor<CS> {
    #[inline]
    fn apply_component_transfer(self, transfer: ComponentTransfer<'_>) -> Self {
        Self::new(transfer.apply_components(self.components))
    }

    #[inline]
    fn apply_color_matrix(self, matrix: ColorMatrix) -> Self {
        Self::new(matrix.apply_components(self.components))
    }
}

impl<CS: color::ColorSpace> ColorOperationTarget for color::PremulColor<CS> {
    #[inline]
    fn apply_component_transfer(self, transfer: ComponentTransfer<'_>) -> Self {
        let color = self.un_premultiply();
        let color = color::AlphaColor::new(transfer.apply_components(color.components));
        color.premultiply()
    }

    #[inline]
    fn apply_color_matrix(self, matrix: ColorMatrix) -> Self {
        let color = self.un_premultiply();
        let color = color::AlphaColor::new(matrix.apply_components(color.components));
        color.premultiply()
    }
}

impl ColorOperationTarget for color::DynamicColor {
    #[inline]
    fn apply_component_transfer(self, transfer: ComponentTransfer<'_>) -> Self {
        dynamic_color_with_components(self, transfer.apply_components(self.components))
    }

    #[inline]
    fn apply_color_matrix(self, matrix: ColorMatrix) -> Self {
        dynamic_color_with_components(self, matrix.apply_components(self.components))
    }
}

#[inline]
fn dynamic_color_with_components(
    color: color::DynamicColor,
    mut components: [f32; 4],
) -> color::DynamicColor {
    let mut flags = color.flags;
    flags.discard_name();

    let missing = flags.missing();
    if !missing.is_empty() {
        for (ix, component) in components.iter_mut().enumerate() {
            if missing.contains(ix) {
                *component = 0.;
            }
        }
    }

    color::DynamicColor {
        cs: color.cs,
        flags,
        components,
    }
}

mod private {
    #[expect(
        unnameable_types,
        reason = "Sealing prevents external implementations."
    )]
    pub trait Sealed {}

    impl<CS: color::ColorSpace> Sealed for color::AlphaColor<CS> {}

    impl<CS: color::ColorSpace> Sealed for color::PremulColor<CS> {}

    impl Sealed for color::DynamicColor {}
}

#[cfg(test)]
mod tests {
    use color::{AlphaColor, ColorSpaceTag, DynamicColor, Flags, Missing, Srgb};

    use crate::{ColorMatrix, ComponentTransfer};

    #[test]
    fn applies_matrix_to_dynamic_color() {
        let color = DynamicColor::from_alpha_color(AlphaColor::<Srgb>::new([0.2, 0.3, 0.4, 0.5]));

        let result = ColorMatrix::brightness(2.).apply(color);

        assert_eq!(result.cs, ColorSpaceTag::Srgb);
        assert_eq!(result.flags, Flags::default());
        assert_eq!(result.components, [0.4, 0.6, 0.8, 0.5]);
    }

    #[test]
    fn applies_component_transfer_to_dynamic_color() {
        let color = DynamicColor::from_alpha_color(AlphaColor::<Srgb>::new([0.25, 0.5, 0.75, 1.]));

        let result = ComponentTransfer::contrast(2.).apply(color);

        assert_eq!(result.cs, ColorSpaceTag::Srgb);
        assert_eq!(result.flags, Flags::default());
        assert_eq!(result.components, [0., 0.5, 1., 1.]);
    }

    #[test]
    fn dynamic_color_preserves_and_zeroes_missing_components() {
        let color = DynamicColor {
            cs: ColorSpaceTag::Srgb,
            flags: Flags::from_missing(Missing::single(1)),
            components: [0.2, 10., 0.4, 0.5],
        };

        let result = ColorMatrix::brightness(2.).apply(color);

        assert_eq!(result.cs, ColorSpaceTag::Srgb);
        assert_eq!(result.flags.missing(), Missing::single(1));
        assert_eq!(result.components, [0.4, 0., 0.8, 0.5]);
    }
}
