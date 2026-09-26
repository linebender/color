// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{ColorMatrix, ColorOperationTarget, ComponentTransfer};

/// A color operation that can be represented by this crate.
///
/// This enum is useful for storing mixed color-operation pipelines without forcing every
/// operation into a matrix representation. It has no color-space semantics. Callers choose the
/// working color space and decide when to convert, clip, or gamut-map colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorOperation<'a> {
    /// A channel-mixing affine matrix operation.
    Matrix(ColorMatrix),
    /// A per-component transfer operation.
    ComponentTransfer(ComponentTransfer<'a>),
}

impl<'a> ColorOperation<'a> {
    /// Create a color operation from a matrix.
    #[inline]
    #[must_use]
    pub const fn matrix(matrix: ColorMatrix) -> Self {
        Self::Matrix(matrix)
    }

    /// Create a color operation from a component transfer.
    #[inline]
    #[must_use]
    pub const fn component_transfer(transfer: ComponentTransfer<'a>) -> Self {
        Self::ComponentTransfer(transfer)
    }

    /// Apply this operation to straight color components.
    #[inline]
    #[must_use]
    pub fn apply_components(self, components: [f32; 4]) -> [f32; 4] {
        match self {
            Self::Matrix(matrix) => matrix.apply_components(components),
            Self::ComponentTransfer(transfer) => transfer.apply_components(components),
        }
    }

    /// Apply this operation to premultiplied color components in a rectangular color space.
    ///
    /// This preserves the same unclipped range policy as [`ColorMatrix`] and
    /// [`ComponentTransfer`].
    #[inline]
    #[must_use]
    pub fn apply_premul_components(self, components: [f32; 4]) -> [f32; 4] {
        match self {
            Self::Matrix(matrix) => matrix.apply_premul_components(components),
            Self::ComponentTransfer(transfer) => transfer.apply_premul_components(components),
        }
    }

    /// Apply this operation to a typed color without converting its color space.
    #[inline]
    #[must_use]
    pub fn apply<C: ColorOperationTarget>(self, color: C) -> C {
        match self {
            Self::Matrix(matrix) => matrix.apply(color),
            Self::ComponentTransfer(transfer) => transfer.apply(color),
        }
    }
}

impl<'a> From<ColorMatrix> for ColorOperation<'a> {
    #[inline]
    fn from(matrix: ColorMatrix) -> Self {
        Self::Matrix(matrix)
    }
}

impl<'a> From<ComponentTransfer<'a>> for ColorOperation<'a> {
    #[inline]
    fn from(transfer: ComponentTransfer<'a>) -> Self {
        Self::ComponentTransfer(transfer)
    }
}

#[cfg(test)]
mod tests {
    use color::{AlphaColor, Srgb};

    use super::ColorOperation;
    use crate::{ColorMatrix, ComponentTransfer};

    #[test]
    fn applies_matrix_operation() {
        let operation = ColorOperation::from(ColorMatrix::brightness(2.));

        assert_eq!(
            operation.apply_components([0.2, 0.3, 0.4, 0.5]),
            [0.4, 0.6, 0.8, 0.5]
        );
    }

    #[test]
    fn applies_component_transfer_operation() {
        let operation = ColorOperation::from(ComponentTransfer::contrast(2.));
        let color = AlphaColor::<Srgb>::new([0.25, 0.5, 0.75, 0.8]);

        assert_eq!(operation.apply(color).components, [0., 0.5, 1., 0.8]);
    }
}
