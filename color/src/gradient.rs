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
/// Use the [`gradient`] function to generate this iterator.
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
/// The target gradient ramp is the linear interpolation from `color0` to `color1` in the target
/// color space specified by `interp_cs`. For efficiency, this function returns an
/// [iterator over color stops](GradientIter) in the `CS` color space, such that the gradient ramp
/// created by linearly interpolating between those stops in the `CS` color space is equal within
/// the specified `tolerance` to the target gradient ramp.
///
/// When the target interpolation color space is cylindrical, the hue can be interpolated in
/// multiple ways. The [`direction`](`HueDirection`) parameter controls the way in which the hue is
/// interpolated.
///
/// The given `tolerance` value specifies the maximum perceptual error in the approximation
/// measured as the [Euclidean distance][euclidean-distance] in the [Oklab] color space (see also
/// [`PremulColor::difference`][crate::PremulColor::difference]). This metric is known as
/// [deltaEOK][delta-eok]. A reasonable value is 0.01, which in testing is nearly indistinguishable
/// from the exact ramp. The number of stops scales roughly as the inverse square root of the
/// tolerance.
///
/// The error is measured at the midpoint of each segment, which in some cases may underestimate
/// the error.
///
/// For regular interpolation between two colors, see [`DynamicColor::interpolate`].
///
/// [euclidean-distance]: https://en.wikipedia.org/wiki/Euclidean_distance
/// [delta-eok]: https://www.w3.org/TR/css-color-4/#color-difference-OK
///
/// # Motivation
///
/// A major feature of CSS Color 4 is the ability to specify color interpolation in any
/// interpolation color space [CSS Color Module Level 4 § 12.1][css-sec], which may be quite a bit
/// better than simple linear interpolation in sRGB (for example).
///
/// One strategy for implementing these gradients is to interpolate in the appropriate
/// (premultiplied) space, then map each resulting color to the space used for compositing. That
/// can be expensive. An alternative strategy is to precompute a piecewise linear ramp that closely
/// approximates the desired ramp, then render that using high performance techniques. This method
/// computes such an approximation.
///
/// [css-sec]: https://www.w3.org/TR/css-color-4/#interpolation-space
///
/// # Example
///
/// The following compares interpolating in the target color space Oklab with interpolating
/// piecewise in the color space sRGB.
///
/// ```rust
/// use color::{AlphaColor, ColorSpaceTag, DynamicColor, HueDirection, Oklab, Srgb};
///
/// let start = DynamicColor::from_alpha_color(AlphaColor::<Srgb>::new([1., 0., 0., 1.]));
/// let end = DynamicColor::from_alpha_color(AlphaColor::<Srgb>::new([0., 1., 0., 1.]));
///
/// // Interpolation in a target interpolation color space.
/// let interp = start.interpolate(end, ColorSpaceTag::Oklab, HueDirection::default());
/// // Piecewise-approximated interpolation in a compositing color space.
/// let mut gradient = color::gradient::<Srgb>(
///     start,
///     end,
///     ColorSpaceTag::Oklab,
///     HueDirection::default(),
///     0.01,
/// );
///
/// let (mut t0, mut stop0) = gradient.next().unwrap();
/// for (t1, stop1) in gradient {
///     // Compare a few points between the piecewise stops.
///     for point in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] {
///         let interpolated_point = interp
///             .eval(t0 + (t1 - t0) * point)
///             .to_alpha_color::<Srgb>()
///             .discard_alpha();
///         let approximated_point = stop0.lerp_rect(stop1, point).discard_alpha();
///
///         // The perceptual deltaEOK between the two is lower than the tolerance.
///         assert!(
///             approximated_point
///                 .convert::<Oklab>()
///                 .difference(interpolated_point.convert::<Oklab>())
///                 < 0.01
///         );
///     }
///
///     t0 = t1;
///     stop0 = stop1;
/// }
/// ```
pub fn gradient<CS: ColorSpace>(
    mut color0: DynamicColor,
    mut color1: DynamicColor,
    interp_cs: ColorSpaceTag,
    direction: HueDirection,
    tolerance: f32,
) -> GradientIter<CS> {
    let interpolator = color0.interpolate(color1, interp_cs, direction);
    if !color0.flags.missing().is_empty() {
        color0 = interpolator.eval(0.0);
    }
    let target0 = color0.to_alpha_color().premultiply();
    if !color1.flags.missing().is_empty() {
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
