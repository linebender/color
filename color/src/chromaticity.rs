// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::{matdiagmatmul, matmatmul, matvecmul};

/// CIE `xy` chromaticity, specifying a color in the XYZ color space, but not its luminosity.
///
/// An absolute color can be specified by adding a luminosity coordinate `Y` as in `xyY`. An `XYZ`
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

    /// Convert the `xy` chromaticities to XYZ, assuming `xyY` with `Y=1`.
    pub(crate) const fn to_xyz(self) -> [f32; 3] {
        let y_recip = 1. / self.y;
        [self.x * y_recip, 1., (1. - self.x - self.y) * y_recip]
    }

    /// Calculate the 3x3 linear Bradford chromatic adaptation matrix from linear sRGB space.
    ///
    /// This calculates the matrix going from a reference white of `self` to a reference white of
    /// `to`.
    pub(crate) const fn linear_srgb_chromatic_adaptation_matrix(self, to: Self) -> [[f32; 3]; 3] {
        let bradford_source = matvecmul(&Self::XYZ_TO_BRADFORD, self.to_xyz());
        let bradford_dest = matvecmul(&Self::XYZ_TO_BRADFORD, to.to_xyz());

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
