// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! CSS colors and syntax.

use crate::{
    color::{add_alpha, fixup_hues_for_interpolate, split_alpha},
    AlphaColor, Bitset, ColorSpace, ColorSpaceLayout, ColorSpaceTag, HueDirection, TaggedColor,
};

#[derive(Clone, Copy, Debug)]
pub struct CssColor {
    pub cs: ColorSpaceTag,
    /// A bitmask of missing components.
    pub missing: Bitset,
    pub components: [f32; 4],
}

#[derive(Clone, Copy)]
#[expect(
    missing_debug_implementations,
    reason = "it's an intermediate struct, only used for eval"
)]
pub struct Interpolator {
    premul1: [f32; 3],
    alpha1: f32,
    delta_premul: [f32; 3],
    alpha2: f32,
    cs: ColorSpaceTag,
    missing: Bitset,
}

impl From<TaggedColor> for CssColor {
    fn from(value: TaggedColor) -> Self {
        Self {
            cs: value.cs,
            missing: Bitset::default(),
            components: value.components,
        }
    }
}

impl CssColor {
    #[must_use]
    pub fn to_tagged_color(self) -> TaggedColor {
        TaggedColor {
            cs: self.cs,
            components: self.components,
        }
    }

    #[must_use]
    pub fn to_alpha_color<CS: ColorSpace>(self) -> AlphaColor<CS> {
        self.to_tagged_color().to_alpha_color()
    }

    #[must_use]
    pub fn from_alpha_color<CS: ColorSpace>(color: AlphaColor<CS>) -> Self {
        TaggedColor::from_alpha_color(color).into()
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
            let tagged = self.to_tagged_color();
            let converted = tagged.convert(cs);
            let mut components = converted.components;
            // Reference: §12.2 of Color 4 spec
            let missing = if self.missing.any() {
                if self.cs.same_analogous(cs) {
                    for (i, component) in components.iter_mut().enumerate() {
                        if self.missing.contains(i) {
                            *component = 0.0;
                        }
                    }
                    self.missing
                } else {
                    let mut missing = self.missing & Bitset::single(3);
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
                Bitset::default()
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

    /// Scale the chroma by the given amount.
    ///
    /// See [`ColorSpace::scale_chroma`] for more details.
    #[must_use]
    pub fn scale_chroma(self, scale: f32) -> Self {
        let (opaque, alpha) = split_alpha(self.components);
        let mut components = self.cs.scale_chroma(opaque, scale);
        if self.missing.any() {
            for (i, component) in components.iter_mut().enumerate() {
                if self.missing.contains(i) {
                    *component = 0.0;
                }
            }
        }
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
            alpha2,
            cs,
            missing,
        }
    }

    /// Compute the relative luminance of the color.
    ///
    /// This can be useful for choosing contrasting colors, and follows the
    /// WCAG 2.1 spec.
    ///
    /// Note that this method only considers the opaque color, not the alpha.
    /// Blending semi-transparent colors will reduce contrast, and that
    /// should also be taken into account.
    pub fn relative_luminance(self) -> f32 {
        let rgb = self.convert(ColorSpaceTag::LinearSrgb).components;
        0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
    }
}

impl Interpolator {
    pub fn eval(&self, t: f32) -> CssColor {
        let premul = [
            self.premul1[0] + t * self.delta_premul[0],
            self.premul1[1] + t * self.delta_premul[1],
            self.premul1[2] + t * self.delta_premul[2],
        ];
        let alpha = self.alpha1 + t * (self.alpha2 - self.alpha1);
        let opaque = if alpha == 0.0 || alpha == 1.0 {
            premul
        } else {
            self.cs.layout().scale(premul, 1.0 / alpha)
        };
        let components = add_alpha(opaque, alpha);
        CssColor {
            cs: self.cs,
            missing: self.missing,
            components,
        }
    }
}
