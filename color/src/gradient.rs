// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{
    ColorSpace, ColorSpaceTag, DynamicColor, HueDirection, Interpolator, Oklab, PremulColor,
};

/// The iterator for gradient approximation.
///
/// This will yield a value for each gradient stop, including `t` values
/// of 0 and 1 at the endpoints.
///
/// Use the `gradient` function to generate this iterator.
#[expect(missing_debug_implementations, reason = "it's an iterator")]
pub struct GradientIter<CS: ColorSpace> {
    interpolator: Interpolator,
    // This is in deltaEOK units
    tolerance: f32,
    // The adaptive subdivision logic is lifted from the stroke expansion paper.
    t0: u32,
    dt: f32,
    target0: PremulColor<CS>,
    target1: PremulColor<CS>,
    end_color: PremulColor<CS>,
}

/// Generate a piecewise linear approximation to a gradient ramp.
///
/// A major feature of CSS Color 4 is to specify gradients in any
/// interpolation color space, which may be quite a bit better than
/// simple linear interpolation in sRGB (for example).
///
/// One strategy for implementing these gradients is to interpolate
/// in the appropriate (premultiplied) space, then map each resulting
/// color to the space used for compositing. That can be expensive.
/// An alternative strategy is to precompute a piecewise linear ramp
/// that closely approximates the desired ramp, then render that
/// using high performance techniques. This method computes such an
/// approximation.
///
/// The given `tolerance` value specifies the maximum error in the
/// approximation, in deltaEOK units. A reasonable value is 0.01,
/// which in testing is nearly indistinguishable from the exact
/// ramp. The number of stops scales roughly as the inverse square
/// root of the tolerance.
///
/// The error is measured at the midpoint of each segment, which in
/// some cases may underestimate the error.
pub fn gradient<CS: ColorSpace>(
    mut color0: DynamicColor,
    mut color1: DynamicColor,
    interp_cs: ColorSpaceTag,
    direction: HueDirection,
    tolerance: f32,
) -> GradientIter<CS> {
    let interpolator = color0.interpolate(color1, interp_cs, direction);
    if !color0.missing.is_empty() {
        color0 = interpolator.eval(0.0);
    }
    let target0 = color0.to_alpha_color().premultiply();
    if !color1.missing.is_empty() {
        color1 = interpolator.eval(1.0);
    }
    let target1 = color1.to_alpha_color().premultiply();
    let end_color = target1;
    GradientIter {
        interpolator,
        tolerance,
        t0: 0,
        dt: 0.0,
        target0,
        target1,
        end_color,
    }
}

impl<CS: ColorSpace> Iterator for GradientIter<CS> {
    type Item = (f32, PremulColor<CS>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.dt == 0.0 {
            self.dt = 1.0;
            return Some((0.0, self.target0));
        }
        let t0 = self.t0 as f32 * self.dt;
        if t0 == 1.0 {
            return None;
        }
        loop {
            // compute midpoint color
            let midpoint = self.interpolator.eval(t0 + 0.5 * self.dt);
            let midpoint_oklab: PremulColor<Oklab> = midpoint.to_alpha_color().premultiply();
            let approx = self.target0.lerp_rect(self.target1, 0.5);
            let error = midpoint_oklab.difference(approx.convert());
            if error <= self.tolerance {
                let t1 = t0 + self.dt;
                self.t0 += 1;
                let shift = self.t0.trailing_zeros();
                self.t0 >>= shift;
                self.dt *= (1 << shift) as f32;
                self.target0 = self.target1;
                let new_t1 = t1 + self.dt;
                if new_t1 < 1.0 {
                    self.target1 = self
                        .interpolator
                        .eval(new_t1)
                        .to_alpha_color()
                        .premultiply();
                } else {
                    self.target1 = self.end_color;
                }
                return Some((t1, self.target0));
            }
            self.t0 *= 2;
            self.dt *= 0.5;
            self.target1 = midpoint.to_alpha_color().premultiply();
        }
    }
}
