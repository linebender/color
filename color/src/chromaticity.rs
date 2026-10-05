// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{OpaqueColor, XyzD65, matdiagmatmul, matmatmul, matvecmul};

/// CIE `xy` chromaticity, specifying a color in the XYZ color space, but not its luminance.
///
/// An absolute color can be specified by adding a luminance coordinate `Y` as in `xyY`. An `XYZ`
/// color can be calculated from `xyY` as follows.
///
/// ```text
/// X = Y/y * x
/// Y = Y
/// Z = Y/y * (1 - x - y)
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chromaticity {
    /// The x-coordinate of the CIE `xy` chromaticity.
    pub x: f32,

    /// The y-coordinate of the CIE `xy` chromaticity.
    pub y: f32,
}

impl Chromaticity {
    /// The CIE D65 white point under the standard 2° observer.
    ///
    /// This is a common white point for color spaces targeting monitors.
    ///
    /// The white point's chromaticities are truncated to four digits here, as specified by the
    /// CSS Color 4 specification, and following most color spaces using this white point.
    pub const D65: Self = Self {
        x: 0.3127,
        y: 0.3290,
    };

    /// The CIE D50 white point under the standard 2° observer.
    ///
    /// The white point's chromaticities are truncated to four digits here, as specified by the
    /// CSS Color 4 specification, and following most color spaces using this white point.
    pub const D50: Self = Self {
        x: 0.3457,
        y: 0.3585,
    };

    /// The [ACES white point][aceswp].
    ///
    /// This is the reference white of [ACEScg](crate::AcesCg) and [ACES2065-1](crate::Aces2065_1).
    /// The white point is near the D60 white point under the standard 2° observer.
    ///
    /// [aceswp]: https://docs.acescentral.com/tb/white-point
    pub const ACES: Self = Self {
        x: 0.32168,
        y: 0.33767,
    };

    /// Approximate the chromaticity of a Planckian radiator at a temperature in kelvin.
    ///
    /// This uses the approximation by [Kang et al.], which is defined for temperatures from
    /// 1667 K through 25,000 K. Returns `None` when `kelvin` is outside that range or is not finite.
    ///
    /// A light source's correlated color temperature (CCT) identifies the nearest point on the
    /// Planckian locus. A real light source may have a chromaticity offset from the value returned
    /// here. CCT also does not specify the source's luminance or spectral distribution.
    ///
    /// # Example
    ///
    /// Convert a 2700 K chromaticity at unit luminance to linear sRGB without chromatic adaptation:
    ///
    /// ```rust
    /// use color::{Chromaticity, ColorSpace, LinearSrgb, XyzD65};
    ///
    /// let chromaticity =
    ///     Chromaticity::try_from_kelvin(2700.).expect("2700 K is within the supported range");
    /// let xyz = chromaticity.with_luminance(1.);
    /// let linear_srgb = XyzD65::convert_absolute::<LinearSrgb>(xyz.components);
    ///
    /// assert!(linear_srgb.iter().all(|component| component.is_finite()));
    /// ```
    ///
    /// [Kang et al.]: https://www.kci.go.kr/kciportal/ci/sereArticleSearch/ciSereArtiView.kci?sereArticleSearchBean.artiId=ART000987865
    #[must_use]
    #[expect(
        clippy::excessive_precision,
        reason = "Preserve Kang et al.'s published coefficients."
    )]
    pub const fn try_from_kelvin(kelvin: f32) -> Option<Self> {
        if !kelvin.is_finite() {
            return None;
        }
        // Keep these as separate conditions so this remains const on the MSRV.
        if kelvin < 1667. {
            return None;
        }
        if kelvin > 25_000. {
            return None;
        }

        // Kang et al. express these polynomials in powers of 1000 / kelvin.
        let t = 1000. / kelvin;
        let t2 = t * t;
        let x = if kelvin <= 4000. {
            0.179_910 + 0.877_695_6 * t - 0.234_358_9 * t2 - 0.266_123_9 * t2 * t
        } else {
            0.240_390 + 0.222_634_7 * t + 2.107_037_9 * t2 - 3.025_846_9 * t2 * t
        };

        let x2 = x * x;
        let y = if kelvin <= 2222. {
            -0.202_196_83 + 2.185_558_32 * x - 1.348_110_20 * x2 - 1.106_381_4 * x2 * x
        } else if kelvin <= 4000. {
            -0.167_488_67 + 2.091_370_15 * x - 1.374_185_93 * x2 - 0.954_947_6 * x2 * x
        } else {
            -0.370_014_83 + 3.751_129_97 * x - 5.873_386_7 * x2 + 3.081_758_0 * x2 * x
        };

        Some(Self { x, y })
    }

    /// Get the color at this chromaticity with the given `luminance`.
    ///
    /// If you convert the color returned by this method to another color space, think carefully
    /// about whether you want to chromatically adapt the color or not. This method returns the
    /// absolute color in [XYZ-D65][`XyzD65`], i.e., encoded with a reference white of [`Chromaticity::D65`].
    /// To get the same absolute color in another color space, use
    /// [`ColorSpace::convert_absolute`][crate::ColorSpace::convert_absolute].
    ///
    /// See the [XYZ-D65](`XyzD65`) color space documentation for some background information on the
    /// meaning of "reference white."
    ///
    /// Note, if [`Self::y`] is zero, the resulting components are non-finite.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use color::{Chromaticity, ColorSpace, OpaqueColor, ProphotoRgb, XyzD65};
    /// // This will be within conversion error of `ProphotoRgb::WHITE_COMPONENTS`,
    /// // i.e., `[1., 1., 1.]`.
    /// let color = XyzD65::convert_absolute::<ProphotoRgb>(
    ///     Chromaticity::D50.with_luminance(1.).components
    /// );
    /// assert!(color.iter().all(|component| (component - 1.).abs() < 1e-4));
    /// ```
    #[inline]
    pub const fn with_luminance(self, luminance: f32) -> OpaqueColor<XyzD65> {
        let y_recip = luminance / self.y;
        OpaqueColor::new([
            self.x * y_recip,
            luminance,
            (1. - self.x - self.y) * y_recip,
        ])
    }

    /// Calculate the 3x3 linear Bradford chromatic adaptation matrix from linear sRGB space.
    ///
    /// This calculates the matrix going from a reference white of `self` to a reference white of
    /// `to`.
    pub(crate) const fn linear_srgb_chromatic_adaptation_matrix(self, to: Self) -> [[f32; 3]; 3] {
        let bradford_source = matvecmul(&Self::XYZ_TO_BRADFORD, self.with_luminance(1.).components);
        let bradford_dest = matvecmul(&Self::XYZ_TO_BRADFORD, to.with_luminance(1.).components);

        matmatmul(
            &matdiagmatmul(
                &Self::BRADFORD_TO_SRGB,
                [
                    bradford_dest[0] / bradford_source[0],
                    bradford_dest[1] / bradford_source[1],
                    bradford_dest[2] / bradford_source[2],
                ],
            ),
            &Self::SRGB_TO_BRADFORD,
        )
    }

    /// `XYZ_to_Bradford * lin_sRGB_to_XYZ`
    const SRGB_TO_BRADFORD: [[f32; 3]; 3] = [
        [
            1_298_421_353. / 3_072_037_500.,
            172_510_403. / 351_090_000.,
            32_024_671. / 1_170_300_000.,
        ],
        [
            85_542_113. / 1_536_018_750.,
            7_089_448_151. / 7_372_890_000.,
            244_246_729. / 10_532_700_000.,
        ],
        [
            131_355_661. / 6_144_075_000.,
            71_798_777. / 819_210_000.,
            3_443_292_119. / 3_510_900_000.,
        ],
    ];

    /// `XYZ_to_lin_sRGB * Bradford_to_XYZ`
    const BRADFORD_TO_SRGB: [[f32; 3]; 3] = [
        [
            3_597_831_250_055_000. / 1_417_335_035_684_489.,
            -1_833_298_161_702_000. / 1_417_335_035_684_489.,
            -57_038_163_791_000. / 1_417_335_035_684_489.,
        ],
        [
            -4_593_417_841_453_000. / 31_461_687_363_220_151.,
            35_130_825_086_032_200. / 31_461_687_363_220_151.,
            -702_492_905_752_400. / 31_461_687_363_220_151.,
        ],
        [
            -191_861_334_350_000. / 4_536_975_728_019_583.,
            -324_802_409_790_000. / 4_536_975_728_019_583.,
            4_639_090_845_380_000. / 4_536_975_728_019_583.,
        ],
    ];

    const XYZ_TO_BRADFORD: [[f32; 3]; 3] = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];
}

#[cfg(test)]
mod tests {
    use crate::{ColorSpace, OpaqueColor, ProphotoRgb, XyzD50, XyzD65};

    use super::Chromaticity;

    #[must_use]
    fn almost_equal<CS: ColorSpace>(col1: [f32; 3], col2: [f32; 3], absolute_epsilon: f32) -> bool {
        OpaqueColor::<CS>::new(col1).difference(OpaqueColor::new(col2)) <= absolute_epsilon
    }

    #[test]
    fn kelvin_domain() {
        assert!(Chromaticity::try_from_kelvin(1667.).is_some());
        assert!(Chromaticity::try_from_kelvin(25_000.).is_some());

        for kelvin in [
            f32::NEG_INFINITY,
            -1.,
            0.,
            1666.9,
            25_000.1,
            f32::INFINITY,
            f32::NAN,
        ] {
            assert!(
                Chromaticity::try_from_kelvin(kelvin).is_none(),
                "{kelvin} K should be outside the supported range"
            );
        }
    }

    #[test]
    fn kelvin_reference_values() {
        // Cross-check the three polynomial regions. The values at 2856 K and above are also used
        // by Android's Kang et al. implementation:
        // https://android.googlesource.com/platform/cts/+/c5e94267c2a3b26e45334c348fae7fab0a7fd04f/tests/tests/graphics/src/android/graphics/cts/ColorSpaceTest.java
        for (kelvin, expected_xyz) in [
            (2000., [1.274_975_5, 1., 0.144_780_1]),
            (2856., [1.097_082_4, 1., 0.356_852_5]),
            (6504., [0.968_573, 1., 1.121_644_4]),
            (24_761., [1.000_648_5, 1., 1.960_453_7]),
        ] {
            let chromaticity = Chromaticity::try_from_kelvin(kelvin)
                .expect("reference temperature should be within the supported range");
            assert!(
                almost_equal::<XyzD65>(
                    chromaticity.with_luminance(1.).components,
                    expected_xyz,
                    1e-6,
                ),
                "unexpected chromaticity at {kelvin} K"
            );
        }
    }

    #[test]
    fn reference_white() {
        assert!(
            almost_equal::<XyzD65>(
                Chromaticity::D65.with_luminance(1.).components,
                XyzD65::WHITE_COMPONENTS,
                1e-4,
            ),
            "`Chromaticity::D65.with_luminance(1)` should match `XyzD65::WHITE_COMPONENTS`"
        );

        assert!(
            almost_equal::<XyzD65>(
                Chromaticity::D50.with_luminance(1.).components,
                XyzD50::WHITE_COMPONENTS,
                1e-4,
            ),
            "`Chromaticity::D50.with_luminance(1)` should match `XyzD50::WHITE_COMPONENTS`"
        );

        assert!(
            almost_equal::<ProphotoRgb>(
                XyzD65::convert_absolute::<ProphotoRgb>(
                    Chromaticity::D50.with_luminance(1.).components
                ),
                ProphotoRgb::WHITE_COMPONENTS,
                1e-4,
            ),
            "`Chromaticity::D50.with_luminance(1)` converted without chromatic adaptation to ProPhoto RGB should match `ProphotoRgb::WHITE_COMPONENTS` (which has a D50 reference white)"
        );
    }
}
