//! Gamut mapping operations
//!
//! There are many possible ways to map colors outside of a color space's natural gamut to colors
//! inside the gamut. The mathematically simplest operation is implemented by [`ColorSpace::clip`].
//! Some methods are perceptually better than others; for example, preserving a color's perceived
//! hue when mapping is generally desirable. Depending on the use-case, other factors may be
//! relevant; for example, when working with an individual color, perhaps it should be mapped to
//! the closest color in the gamut. In contrast, when compressing high dynamic range photographs
//! into a gamut, perhaps the relationship between colors is more important than sticking as close
//! as possible to the original colors.
//!
//! # Examples
//!
//! ```rust
//! use color::{gamut_map, ColorSpace, LinearSrgb};
//!
//! // A linear sRGB color with the red color component outside of the natural gamut.
//! let color = [1.1, 0.5, 0.0];
//! assert_ne!(LinearSrgb::clip(color), color);
//!
//! let mapped = gamut_map::reduce_chroma::<LinearSrgb>(color, 0.02);
//!
//! // The mapped color is inside the gamut.
//! assert_eq!(LinearSrgb::clip(mapped), mapped);
//! ```

use crate::{ColorSpace, ColorSpaceTag, Oklab, Oklch};

/// Fits `src` into the natural gamut of the color space, under a relative colorimetric rendering
/// intent, by reducing the color's chroma in the [`Oklch`] color space.
///
/// This works on individual colors. When used to map multiple colors into the color space's gamut,
/// the relationship between those colors may become distorted.
///
/// The color's chroma is reduced until the [clipped](ColorSpace::clip) color (which always fits
/// inside the gamut) is *not noticeably different* from the current chroma-reduced color. This
/// helps prevent excessive chroma reduction that might otherwise result due to the concativity of
/// the gamut boundary. Colors are not noticeably different if their *DeltaEOK* is less than
/// `jnd`.
///
/// A common value for `jnd` is 0.02.
pub fn reduce_chroma<CS: ColorSpace>(src: [f32; 3], jnd: f32) -> [f32; 3] {
    // This implements the binary search gamut-finding algorithm from CSS Color Module 4. See:
    // https://www.w3.org/TR/css-color-4/#binsearch
    const EPSILON: f32 = 0.000_1;

    /// DeltaEOK squared between a color in `CS` space and `Oklch` space
    fn delta_eok2<CS: ColorSpace>(cs: [f32; 3], oklch: [f32; 3]) -> f32 {
        let src1 = CS::convert::<Oklab>(cs);
        let src2 = Oklch::convert::<Oklab>(oklch);
        (src1[0] - src2[0]).powi(2) + (src1[1] - src2[1]).powi(2) + (src1[2] - src2[2]).powi(2)
    }

    // Short-circuit unbounded color spaces.
    if matches!(
        CS::TAG,
        Some(ColorSpaceTag::Oklch | ColorSpaceTag::Oklab | ColorSpaceTag::XyzD65)
    ) {
        return src;
    }

    debug_assert!(jnd > 0.);
    let jnd2 = jnd * jnd;

    // The current color in Oklch.
    let [l, mut c, h] = CS::convert::<Oklch>(src);

    if l < 0. {
        return Oklch::convert::<CS>([0., 0., 0.]);
    } else if l > 1. {
        return Oklch::convert::<CS>([1., 0., 0.]);
    }

    // The clipped color in CS.
    let mut clipped = CS::clip(src);

    if delta_eok2::<CS>(clipped, [l, c, h]) < jnd2 {
        return clipped;
    }

    let mut min = 0.;
    let mut max = c;
    let mut min_in_gamut = true;

    while max - min > EPSILON {
        c = 0.5 * (min + max);
        let current_cs = Oklch::convert::<CS>([l, c, h]);
        let clipped_ = CS::clip(current_cs);

        if min_in_gamut && clipped_ == current_cs {
            min = c;
            continue;
        }

        clipped = clipped_;
        let err2 = delta_eok2::<CS>(clipped, [l, c, h]);
        if err2 < jnd2 {
            if jnd2 - err2 < EPSILON * EPSILON {
                return clipped;
            } else {
                min_in_gamut = false;
                min = c;
            }
        } else {
            max = c;
        }
    }

    clipped
}

#[cfg(test)]
mod tests {
    use crate::{ColorSpace, Oklab, Oklch, Srgb};

    use super::reduce_chroma;

    fn deltaeok<CS: ColorSpace>(src1: [f32; 3], src2: [f32; 3]) -> f32 {
        let src1 = CS::convert::<Oklab>(src1);
        let src2 = CS::convert::<Oklab>(src2);

        ((src1[0] - src2[0]).powi(2) + (src1[1] - src2[1]).powi(2) + (src1[2] - src2[2]).powi(2))
            .sqrt()
    }

    #[test]
    fn reduce_chroma_roundtrip_in_gamut() {
        const EPSILON: f32 = 0.000_000_1;

        let components = [0.0, 1.0, 0.5, 0.001, 0.999];
        for r in components {
            for g in components {
                for b in components {
                    let color = [r, g, b];
                    let mapped = reduce_chroma::<Srgb>(color, 0.002);

                    // The original color must be returned modulo roundoff errors.
                    assert!(deltaeok::<Srgb>(color, mapped) < EPSILON);

                    // The mapped color must still be inside the gamut (and not be nudged out,
                    // e.g., due to numerical stability).
                    assert_eq!(Srgb::clip(mapped), color);
                }
            }
        }
    }

    #[test]
    fn reduce_chroma_known_reference() {
        // Add some more reference values
        let srgb = Oklch::convert::<Srgb>([0.5, 0.205, 230.]);
        let color = reduce_chroma::<Srgb>(srgb, 0.02);
        assert!(deltaeok::<Srgb>(color, [0., 109. / 255., 145. / 255.]) < 0.02);
    }
}
