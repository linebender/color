// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Concrete types for colors.

use core::any::TypeId;
use core::marker::PhantomData;

use crate::{ColorSpace, ColorSpaceLayout};

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

/// An opaque color.
///
/// A color in a color space known at compile time, without transparency. Note
/// that "opaque" refers to the color, not the representation; the components
/// are publicly accessible.
///
/// Arithmetic traits are defined on this type, and operate component-wise. A
/// major motivation for including these is to enable weighted sums, including
/// for spline interpolation. For cylindrical color spaces, hue fixup should
/// be applied before interpolation.
#[derive(Clone, Copy, Debug)]
pub struct OpaqueColor<CS> {
    pub components: [f32; 3],
    pub cs: PhantomData<CS>,
}

/// A color with an alpha channel.
///
/// A color in a color space known at compile time, with an alpha channel.
///
/// See [`OpaqueColor`] for a discussion of arithmetic traits and interpolation.
#[derive(Clone, Copy, Debug)]
pub struct AlphaColor<CS> {
    pub components: [f32; 4],
    pub cs: PhantomData<CS>,
}

/// A color with premultiplied alpha.
///
/// A color in a color space known at compile time, with a premultiplied
/// alpha channel.
///
/// Following the convention of CSS Color 4, in cylindrical color spaces
/// the hue channel is not premultiplied. If it were, interpolation would
/// give undesirable results.
///
/// See [`OpaqueColor`] for a discussion of arithmetic traits and interpolation.
#[derive(Clone, Copy, Debug)]
pub struct PremulColor<CS> {
    pub components: [f32; 4],
    pub cs: PhantomData<CS>,
}

/// The hue direction for interpolation.
///
/// This type corresponds to [`hue-interpolation-method`] in the CSS Color
/// 4 spec.
///
/// [`hue-interpolation-method`]: https://developer.mozilla.org/en-US/docs/Web/CSS/hue-interpolation-method
#[derive(Clone, Copy, Default, Debug)]
#[non_exhaustive]
pub enum HueDirection {
    #[default]
    Shorter,
    Longer,
    Increasing,
    Decreasing,
    // It's possible we'll add "raw"; color.js has it.
}

/// Fixup hue based on specified hue direction.
///
/// Reference: §12.4 of CSS Color 4 spec
///
/// Note that this technique has been tweaked to only modify the second hue.
/// The rationale for this is to support multiple gradient stops, for example
/// in a spline. Apply the fixup to successive adjacent pairs.
///
/// In addition, hues outside [0, 360) are supported, with a resulting hue
/// difference always in [-360, 360].
fn fixup_hue(h1: f32, h2: &mut f32, direction: HueDirection) {
    let dh = (*h2 - h1) * (1. / 360.);
    match direction {
        HueDirection::Shorter => {
            // Round, resolving ties toward zero.
            let rounded = if dh - dh.floor() == 0.5 {
                dh.trunc()
            } else {
                dh.round()
            };
            *h2 -= 360. * rounded;
        }
        HueDirection::Longer => {
            let t = 2.0 * dh.abs().ceil() - (dh.abs() + 1.5).floor();
            *h2 += 360.0 * (t.copysign(0.0 - dh));
        }
        HueDirection::Increasing => *h2 -= 360.0 * dh.floor(),
        HueDirection::Decreasing => *h2 -= 360.0 * dh.ceil(),
    }
}

pub(crate) fn fixup_hues_for_interpolate(
    a: [f32; 3],
    b: &mut [f32; 3],
    layout: ColorSpaceLayout,
    direction: HueDirection,
) {
    if let Some(ix) = layout.hue_channel() {
        fixup_hue(a[ix], &mut b[ix], direction);
    }
}

impl<CS: ColorSpace> OpaqueColor<CS> {
    pub const fn new(components: [f32; 3]) -> Self {
        let cs = PhantomData;
        Self { components, cs }
    }

    pub fn convert<TargetCS: ColorSpace>(self) -> OpaqueColor<TargetCS> {
        OpaqueColor::new(CS::convert::<TargetCS>(self.components))
    }

    /// Add an alpha channel.
    ///
    /// This function is the inverse of [`AlphaColor::split`].
    pub const fn with_alpha(self, alpha: f32) -> AlphaColor<CS> {
        AlphaColor::new(add_alpha(self.components, alpha))
    }

    /// Difference between two colors by Euclidean metric.
    pub fn difference(self, other: Self) -> f32 {
        let x = self.components;
        let y = other.components;
        let (d0, d1, d2) = (x[0] - y[0], x[1] - y[1], x[2] - y[2]);
        (d0 * d0 + d1 * d1 + d2 * d2).sqrt()
    }

    /// Linearly interpolate colors, without hue fixup.
    ///
    /// This method produces meaningful results in rectangular color spaces,
    /// or if hue fixup has been applied.
    #[must_use]
    pub fn lerp_rect(self, other: Self, t: f32) -> Self {
        self + t * (other - self)
    }

    /// Apply hue fixup for interpolation.
    ///
    /// Adjust the hue angle of `other` so that linear interpolation results in
    /// the expected hue direction.
    pub fn fixup_hues(self, other: &mut Self, direction: HueDirection) {
        fixup_hues_for_interpolate(
            self.components,
            &mut other.components,
            CS::LAYOUT,
            direction,
        );
    }

    /// Linearly interpolate colors, with hue fixup if needed.
    #[must_use]
    pub fn lerp(self, mut other: Self, t: f32, direction: HueDirection) -> Self {
        self.fixup_hues(&mut other, direction);
        self.lerp_rect(other, t)
    }

    /// Scale the chroma by the given amount.
    ///
    /// See [`ColorSpace::scale_chroma`] for more details.
    #[must_use]
    pub fn scale_chroma(self, scale: f32) -> Self {
        Self::new(CS::scale_chroma(self.components, scale))
    }

    /// Compute the relative luminance of the color.
    ///
    /// This can be useful for choosing contrasting colors, and follows the
    /// WCAG 2.1 spec.
    pub fn relative_luminance(self) -> f32 {
        let rgb = CS::to_linear_srgb(self.components);
        0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
    }
}

pub(crate) const fn split_alpha([x, y, z, a]: [f32; 4]) -> ([f32; 3], f32) {
    ([x, y, z], a)
}

pub(crate) const fn add_alpha([x, y, z]: [f32; 3], a: f32) -> [f32; 4] {
    [x, y, z, a]
}

impl<CS: ColorSpace> AlphaColor<CS> {
    pub const fn new(components: [f32; 4]) -> Self {
        let cs = PhantomData;
        Self { components, cs }
    }

    /// Split into opaque and alpha components.
    ///
    /// This function is the inverse of [`OpaqueColor::with_alpha`].
    #[must_use]
    pub const fn split(self) -> (OpaqueColor<CS>, f32) {
        let (opaque, alpha) = split_alpha(self.components);
        (OpaqueColor::new(opaque), alpha)
    }

    #[must_use]
    pub fn convert<TargetCs: ColorSpace>(self) -> AlphaColor<TargetCs> {
        let (opaque, alpha) = split_alpha(self.components);
        let components = CS::convert::<TargetCs>(opaque);
        AlphaColor::new(add_alpha(components, alpha))
    }

    #[must_use]
    pub const fn premultiply(self) -> PremulColor<CS> {
        let (opaque, alpha) = split_alpha(self.components);
        PremulColor::new(add_alpha(CS::LAYOUT.scale(opaque, alpha), alpha))
    }

    #[must_use]
    pub fn lerp_rect(self, other: Self, t: f32) -> Self {
        self.premultiply()
            .lerp_rect(other.premultiply(), t)
            .un_premultiply()
    }

    #[must_use]
    pub fn lerp(self, other: Self, t: f32, direction: HueDirection) -> Self {
        self.premultiply()
            .lerp(other.premultiply(), t, direction)
            .un_premultiply()
    }

    #[must_use]
    pub const fn mul_alpha(self, rhs: f32) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        Self::new(add_alpha(opaque, alpha * rhs))
    }

    /// Scale the chroma by the given amount.
    ///
    /// See [`ColorSpace::scale_chroma`] for more details.
    #[must_use]
    pub fn scale_chroma(self, scale: f32) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        Self::new(add_alpha(CS::scale_chroma(opaque, scale), alpha))
    }
}

impl<CS: ColorSpace> PremulColor<CS> {
    pub const fn new(components: [f32; 4]) -> Self {
        let cs = PhantomData;
        Self { components, cs }
    }

    #[must_use]
    pub fn convert<TargetCS: ColorSpace>(self) -> PremulColor<TargetCS> {
        if TypeId::of::<CS>() == TypeId::of::<TargetCS>() {
            PremulColor::new(self.components)
        } else if TargetCS::IS_LINEAR && CS::IS_LINEAR {
            let (multiplied, alpha) = split_alpha(self.components);
            let components = CS::convert::<TargetCS>(multiplied);
            PremulColor::new(add_alpha(components, alpha))
        } else {
            self.un_premultiply().convert().premultiply()
        }
    }

    #[must_use]
    pub fn un_premultiply(self) -> AlphaColor<CS> {
        let (multiplied, alpha) = split_alpha(self.components);
        let scale = if alpha == 0.0 { 1.0 } else { 1.0 / alpha };
        AlphaColor::new(add_alpha(CS::LAYOUT.scale(multiplied, scale), alpha))
    }

    /// Interpolate colors.
    ///
    /// Note: this function doesn't fix up hue in cylindrical spaces. It is
    /// still useful if the hue angles are compatible, particularly if the
    /// fixup has been applied.
    #[must_use]
    pub fn lerp_rect(self, other: Self, t: f32) -> Self {
        self + t * (other - self)
    }

    /// Apply hue fixup for interpolation.
    ///
    /// Adjust the hue angle of `other` so that linear interpolation results in
    /// the expected hue direction.
    pub fn fixup_hues(self, other: &mut Self, direction: HueDirection) {
        if let Some(ix) = CS::LAYOUT.hue_channel() {
            fixup_hue(self.components[ix], &mut other.components[ix], direction);
        }
    }

    /// Linearly interpolate colors, with hue fixup if needed.
    #[must_use]
    pub fn lerp(self, mut other: Self, t: f32, direction: HueDirection) -> Self {
        self.fixup_hues(&mut other, direction);
        self.lerp_rect(other, t)
    }

    #[must_use]
    pub const fn mul_alpha(self, rhs: f32) -> Self {
        let (multiplied, alpha) = split_alpha(self.components);
        Self::new(add_alpha(CS::LAYOUT.scale(multiplied, rhs), alpha * rhs))
    }

    /// Difference between two colors by Euclidean metric.
    #[must_use]
    pub fn difference(self, other: Self) -> f32 {
        let d = (self - other).components;
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2] + d[3] * d[3]).sqrt()
    }
}

// Lossless conversion traits.

impl<CS: ColorSpace> From<OpaqueColor<CS>> for AlphaColor<CS> {
    fn from(value: OpaqueColor<CS>) -> Self {
        value.with_alpha(1.0)
    }
}

impl<CS: ColorSpace> From<OpaqueColor<CS>> for PremulColor<CS> {
    fn from(value: OpaqueColor<CS>) -> Self {
        Self::new(add_alpha(value.components, 1.0))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<f32> for OpaqueColor<CS> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.components.map(|x| x * rhs))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<OpaqueColor<CS>> for f32 {
    type Output = OpaqueColor<CS>;

    fn mul(self, rhs: OpaqueColor<CS>) -> Self::Output {
        rhs * self
    }
}

/// Divide components by a scalar.
impl<CS: ColorSpace> core::ops::Div<f32> for OpaqueColor<CS> {
    type Output = Self;

    // https://github.com/rust-lang/rust-clippy/issues/13652 has been filed
    #[expect(clippy::suspicious_arithmetic_impl, reason = "multiplicative inverse")]
    fn div(self, rhs: f32) -> Self {
        self * rhs.recip()
    }
}

/// Component-wise addition of components.
impl<CS: ColorSpace> core::ops::Add for OpaqueColor<CS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] + y[0], x[1] + y[1], x[2] + y[2]])
    }
}

/// Component-wise subtraction of components.
impl<CS: ColorSpace> core::ops::Sub for OpaqueColor<CS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] - y[0], x[1] - y[1], x[2] - y[2]])
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<f32> for AlphaColor<CS> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.components.map(|x| x * rhs))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<AlphaColor<CS>> for f32 {
    type Output = AlphaColor<CS>;

    fn mul(self, rhs: AlphaColor<CS>) -> Self::Output {
        rhs * self
    }
}

/// Divide components by a scalar.
impl<CS: ColorSpace> core::ops::Div<f32> for AlphaColor<CS> {
    type Output = Self;

    #[expect(clippy::suspicious_arithmetic_impl, reason = "multiplicative inverse")]
    fn div(self, rhs: f32) -> Self {
        self * rhs.recip()
    }
}

/// Component-wise addition of components.
impl<CS: ColorSpace> core::ops::Add for AlphaColor<CS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] + y[0], x[1] + y[1], x[2] + y[2], x[3] + y[3]])
    }
}

/// Component-wise subtraction of components.
impl<CS: ColorSpace> core::ops::Sub for AlphaColor<CS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] - y[0], x[1] - y[1], x[2] - y[2], x[3] - y[3]])
    }
}

/// Multiply components by a scalar.
///
/// For rectangular color spaces, this is equivalent to multiplying
/// alpha, but for cylindrical color spaces, [`PremulColor::mul_alpha`]
/// is the preferred method.
impl<CS: ColorSpace> core::ops::Mul<f32> for PremulColor<CS> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self::new(self.components.map(|x| x * rhs))
    }
}

/// Multiply components by a scalar.
impl<CS: ColorSpace> core::ops::Mul<PremulColor<CS>> for f32 {
    type Output = PremulColor<CS>;

    fn mul(self, rhs: PremulColor<CS>) -> Self::Output {
        rhs * self
    }
}

/// Divide components by a scalar.
impl<CS: ColorSpace> core::ops::Div<f32> for PremulColor<CS> {
    type Output = Self;

    #[expect(clippy::suspicious_arithmetic_impl, reason = "multiplicative inverse")]
    fn div(self, rhs: f32) -> Self {
        self * rhs.recip()
    }
}

/// Component-wise addition of components.
impl<CS: ColorSpace> core::ops::Add for PremulColor<CS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] + y[0], x[1] + y[1], x[2] + y[2], x[3] + y[3]])
    }
}

/// Component-wise subtraction of components.
impl<CS: ColorSpace> core::ops::Sub for PremulColor<CS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let x = self.components;
        let y = rhs.components;
        Self::new([x[0] - y[0], x[1] - y[1], x[2] - y[2], x[3] - y[3]])
    }
}

#[cfg(test)]
mod tests {
    use super::{fixup_hue, HueDirection};

    #[test]
    fn hue_fixup() {
        // Verify that the hue arc matches the spec for all hues specified
        // within [0,360).
        for h1 in [0.0, 10.0, 180.0, 190.0, 350.0] {
            for h2 in [0.0, 10.0, 180.0, 190.0, 350.0] {
                let dh = h2 - h1;
                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Shorter);
                    let (mut spec_h1, mut spec_h2) = (h1, h2);
                    if dh > 180.0 {
                        spec_h1 += 360.0;
                    } else if dh < -180.0 {
                        spec_h2 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }

                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Longer);
                    let (mut spec_h1, mut spec_h2) = (h1, h2);
                    if 0.0 < dh && dh < 180.0 {
                        spec_h1 += 360.0;
                    } else if -180.0 < dh && dh <= 0.0 {
                        spec_h2 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }

                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Increasing);
                    let (spec_h1, mut spec_h2) = (h1, h2);
                    if dh < 0.0 {
                        spec_h2 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }

                {
                    let mut fixed_h2 = h2;
                    fixup_hue(h1, &mut fixed_h2, HueDirection::Decreasing);
                    let (mut spec_h1, spec_h2) = (h1, h2);
                    if dh > 0.0 {
                        spec_h1 += 360.0;
                    }
                    assert_eq!(fixed_h2 - h1, spec_h2 - spec_h1);
                }
            }
        }
    }
}