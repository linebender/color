// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CSS colors and syntax.

use crate::{
    color::{add_alpha, fixup_hues_for_interpolate, split_alpha},
    AlphaColor, ColorSpace, ColorSpaceLayout, ColorSpaceTag, HueDirection, LinearSrgb, Missing,
};

/// A color with a color space tag decided at runtime.
///
/// This type is roughly equivalent to [`AlphaColor`] except with a tag
/// for color space as opposed being determined at compile time. It can
/// also represent missing components, which are a feature of the CSS
/// Color 4 spec.
///
/// Missing components are mostly useful for interpolation, and in that
/// context take the value of the other color being interpolated. For
/// example, interpolating a color in [Oklch] with `oklch(none 0 none)`
/// fades the color saturation, ending in a gray with the same lightness.
///
/// In other contexts, missing colors are interpreted as a zero value.
/// When manipulating components directly, setting them nonzero when the
/// corresponding missing flag is set may yield unexpected results.
///
/// [Oklch]: crate::Oklch
#[derive(Clone, Copy, Debug)]
pub struct DynamicColor {
    /// The color space.
    pub cs: ColorSpaceTag,
    /// A bitmask of missing components.
    pub missing: Missing,
    /// The components.
    ///
    /// The first three components are interpreted according to the
    /// color space tag. The fourth component is alpha, interpreted
    /// as separate alpha.
    pub components: [f32; 4],
}

/// An intermediate struct used for interpolating between colors.
///
/// This is the return value of [`DynamicColor::interpolate`].
#[derive(Clone, Copy)]
#[expect(
    missing_debug_implementations,
    reason = "it's an intermediate struct, only used for eval"
)]
pub struct Interpolator {
    premul1: [f32; 3],
    alpha1: f32,
    delta_premul: [f32; 3],
    delta_alpha: f32,
    cs: ColorSpaceTag,
    missing: Missing,
}

impl DynamicColor {
    /// Convert to `AlphaColor` with a static color space.
    ///
    /// Missing components are interpreted as 0.
    #[must_use]
    pub fn to_alpha_color<CS: ColorSpace>(self) -> AlphaColor<CS> {
        if let Some(cs) = CS::TAG {
            AlphaColor::new(self.convert(cs).components)
        } else {
            self.to_alpha_color::<LinearSrgb>().convert()
        }
    }

    /// Convert from `AlphaColor`.
    #[must_use]
    pub fn from_alpha_color<CS: ColorSpace>(color: AlphaColor<CS>) -> Self {
        if let Some(cs) = CS::TAG {
            Self {
                cs,
                missing: Missing::default(),
                components: color.components,
            }
        } else {
            Self::from_alpha_color(color.convert::<LinearSrgb>())
        }
    }

    #[must_use]
    /// Convert to a different color space.
    pub fn convert(self, cs: ColorSpaceTag) -> Self {
        if self.cs == cs {
            // Note: §12 suggests that changing powerless to missing happens
            // even when the color is already in the interpolation color space,
            // but Chrome and color.js don't seem do to that.
            self
        } else {
            let (opaque, alpha) = split_alpha(self.components);
            let mut components = add_alpha(self.cs.convert(cs, opaque), alpha);
            // Reference: §12.2 of Color 4 spec
            let missing = if !self.missing.is_empty() {
                if self.cs.same_analogous(cs) {
                    for (i, component) in components.iter_mut().enumerate() {
                        if self.missing.contains(i) {
                            *component = 0.0;
                        }
                    }
                    self.missing
                } else {
                    let mut missing = self.missing & Missing::single(3);
                    if self.cs.h_missing(self.missing) {
                        cs.set_h_missing(&mut missing, &mut components);
                    }
                    if self.cs.c_missing(self.missing) {
                        cs.set_c_missing(&mut missing, &mut components);
                    }
                    if self.cs.l_missing(self.missing) {
                        cs.set_l_missing(&mut missing, &mut components);
                    }
                    missing
                }
            } else {
                Missing::default()
            };
            let mut result = Self {
                cs,
                missing,
                components,
            };
            result.powerless_to_missing();
            result
        }
    }

    /// Set any missing components to zero.
    ///
    /// We have a soft invariant that any bit set in the missing bitflag has
    /// a corresponding component which is 0. This method restores that
    /// invariant after manipulation which might invalidate it.
    fn zero_missing_components(mut self) -> Self {
        if !self.missing.is_empty() {
            for (i, component) in self.components.iter_mut().enumerate() {
                if self.missing.contains(i) {
                    *component = 0.0;
                }
            }
        }
        self
    }

    /// Scale the chroma by the given amount.
    ///
    /// See [`ColorSpace::scale_chroma`] for more details.
    #[must_use]
    pub fn scale_chroma(self, scale: f32) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        let components = self.cs.scale_chroma(opaque, scale);
        Self {
            cs: self.cs,
            missing: self.missing,
            components: add_alpha(components, alpha),
        }
        .zero_missing_components()
    }

    /// Clip the color's components to fit within the natural gamut of the color space, and clamp
    /// the color's alpha to be in the range `[0, 1]`.
    ///
    /// See [`ColorSpace::clip`] for more details.
    #[must_use]
    pub fn clip(self) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        let components = self.cs.clip(opaque);
        let alpha = alpha.clamp(0., 1.);
        Self {
            cs: self.cs,
            missing: self.missing,
            components: add_alpha(components, alpha),
        }
    }

    fn premultiply_split(self) -> ([f32; 3], f32) {
        // Reference: §12.3 of Color 4 spec
        let (opaque, alpha) = split_alpha(self.components);
        let premul = if alpha == 1.0 || self.missing.contains(3) {
            opaque
        } else {
            self.cs.layout().scale(opaque, alpha)
        };
        (premul, alpha)
    }

    fn powerless_to_missing(&mut self) {
        // Note: the spec seems vague on the details of what this should do,
        // and there is some controversy in discussion threads. For example,
        // in Lab-like spaces, if L is 0 do the other components become powerless?
        const POWERLESS_EPSILON: f32 = 1e-6;
        if self.cs.layout() != ColorSpaceLayout::Rectangular
            && self.components[1] < POWERLESS_EPSILON
        {
            self.cs
                .set_h_missing(&mut self.missing, &mut self.components);
        }
    }

    /// Interpolate two colors, according to CSS Color 4 spec.
    ///
    /// This method does a bunch of precomputation, resulting in an [`Interpolator`]
    /// object that can be evaluated at various `t` values.
    ///
    /// Reference: §12 of Color 4 spec
    pub fn interpolate(
        self,
        other: Self,
        cs: ColorSpaceTag,
        direction: HueDirection,
    ) -> Interpolator {
        let mut a = self.convert(cs);
        let mut b = other.convert(cs);
        let missing = a.missing & b.missing;
        if self.missing != other.missing {
            for i in 0..4 {
                if (a.missing & !b.missing).contains(i) {
                    a.components[i] = b.components[i];
                } else if (!a.missing & b.missing).contains(i) {
                    b.components[i] = a.components[i];
                }
            }
        }
        let (premul1, alpha1) = a.premultiply_split();
        let (mut premul2, alpha2) = b.premultiply_split();
        fixup_hues_for_interpolate(premul1, &mut premul2, cs.layout(), direction);
        let delta_premul = [
            premul2[0] - premul1[0],
            premul2[1] - premul1[1],
            premul2[2] - premul1[2],
        ];
        Interpolator {
            premul1,
            alpha1,
            delta_premul,
            delta_alpha: alpha2 - alpha1,
            cs,
            missing,
        }
    }

    /// Compute the relative luminance of the color.
    ///
    /// This can be useful for choosing contrasting colors, and follows the
    /// [WCAG 2.1 spec].
    ///
    /// Note that this method only considers the opaque color, not the alpha.
    /// Blending semi-transparent colors will reduce contrast, and that
    /// should also be taken into account.
    ///
    /// [WCAG 2.1 spec]: https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
    #[must_use]
    pub fn relative_luminance(self) -> f32 {
        let [r, g, b, _] = self.convert(ColorSpaceTag::LinearSrgb).components;
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Map components.
    #[must_use]
    pub fn map(self, f: impl Fn(f32, f32, f32, f32) -> [f32; 4]) -> Self {
        let [x, y, z, a] = self.components;
        Self {
            cs: self.cs,
            missing: self.missing,
            components: f(x, y, z, a),
        }
        .zero_missing_components()
    }

    /// Map components in a given color space.
    #[must_use]
    pub fn map_in(self, cs: ColorSpaceTag, f: impl Fn(f32, f32, f32, f32) -> [f32; 4]) -> Self {
        self.convert(cs).map(f).convert(self.cs)
    }

    /// Map the lightness of the color.
    ///
    /// In a color space that naturally has a lightness component, map that value.
    /// Otherwise, do the mapping in [Oklab]. The lightness range is normalized so
    /// that 1.0 is white. That is the normal range for Oklab but differs from the
    /// range in [Lab], [Lch], and [Hsl].
    ///
    /// [Oklab]: crate::Oklab
    /// [Lab]: crate::Lab
    /// [Lch]: crate::Lch
    /// [Hsl]: crate::Hsl
    #[must_use]
    pub fn map_lightness(self, f: impl Fn(f32) -> f32) -> Self {
        match self.cs {
            ColorSpaceTag::Lab | ColorSpaceTag::Lch => {
                self.map(|l, c1, c2, a| [100.0 * f(l * 0.01), c1, c2, a])
            }
            ColorSpaceTag::Oklab | ColorSpaceTag::Oklch => {
                self.map(|l, c1, c2, a| [f(l), c1, c2, a])
            }
            ColorSpaceTag::Hsl => self.map(|h, s, l, a| [h, s, 100.0 * f(l * 0.01), a]),
            _ => self.map_in(ColorSpaceTag::Oklab, |l, a, b, alpha| [f(l), a, b, alpha]),
        }
    }

    /// Map the hue of the color.
    ///
    /// In a color space that naturally has a hue component, map that value.
    /// Otherwise, do the mapping in [Oklch]. The hue is in degrees.
    ///
    /// [Oklch]: crate::Oklch
    #[must_use]
    pub fn map_hue(self, f: impl Fn(f32) -> f32) -> Self {
        match self.cs.layout() {
            ColorSpaceLayout::HueFirst => self.map(|h, c1, c2, a| [f(h), c1, c2, a]),
            ColorSpaceLayout::HueThird => self.map(|c0, c1, h, a| [c0, c1, f(h), a]),
            _ => self.map_in(ColorSpaceTag::Oklch, |l, c, h, a| [l, c, f(h), a]),
        }
    }
}

impl Interpolator {
    /// Evaluate the color ramp at the given point.
    ///
    /// Typically `t` ranges between 0 and 1, but that is not enforced,
    /// so extrapolation is also possible.
    pub fn eval(&self, t: f32) -> DynamicColor {
        let premul = [
            self.premul1[0] + t * self.delta_premul[0],
            self.premul1[1] + t * self.delta_premul[1],
            self.premul1[2] + t * self.delta_premul[2],
        ];
        let alpha = self.alpha1 + t * self.delta_alpha;
        let opaque = if alpha == 0.0 || alpha == 1.0 {
            premul
        } else {
            self.cs.layout().scale(premul, 1.0 / alpha)
        };
        let components = add_alpha(opaque, alpha);
        DynamicColor {
            cs: self.cs,
            missing: self.missing,
            components,
        }
    }
}
