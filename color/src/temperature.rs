// Copyright 2025 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![expect(
    clippy::excessive_precision,
    reason = "We are sticking with the constants from Tanner Helland."
)]

use crate::{AlphaColor, ColorSpace, OpaqueColor};

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

// Implementation notes:
//
// * These can't be const yet as `clamp`, `ln`, and `powf` are not `const`.

impl<CS: ColorSpace> AlphaColor<CS> {
    /// Convert a temperature in Kelvin to a color.
    ///
    /// The Kelvin temperature will be clamped to a range of 1000-40,000.
    pub fn from_kelvin(kelvin: f32) -> Self {
        let [r, g, b] = CS::from_linear_srgb(from_kelvin(kelvin));
        Self::new([r, g, b, 1.])
    }
}

impl<CS: ColorSpace> OpaqueColor<CS> {
    /// Convert a temperature in Kelvin to a color.
    ///
    /// The Kelvin temperature will be clamped to a range of 1000-40,000.
    pub fn from_kelvin(kelvin: f32) -> Self {
        Self::new(CS::from_linear_srgb(from_kelvin(kelvin)))
    }
}

/// Convert a Kelvin temperature into a linear RGB color.
fn from_kelvin(kelvin: f32) -> [f32; 3] {
    let kelvin = kelvin.clamp(1000., 40000.) / 100.;

    let red = if kelvin < 66. {
        255.
    } else {
        329.698_727_446 * (kelvin - 60.).powf(-0.133_204_759_2)
    };

    let green = if kelvin <= 66. {
        99.470_802_586_1 * kelvin.ln() - 161.119_568_166_1
    } else {
        288.122_169_528_3 * (kelvin - 60.).powf(-0.075_514_849_2)
    };

    let blue = if kelvin >= 66. {
        255.
    } else if kelvin <= 19. {
        0.
    } else {
        138.517_731_223_1 * (kelvin - 10.).ln() - 305.044_792_730_7
    };

    [
        red.min(255.) / 255.,
        green.min(255.) / 255.,
        blue.min(255.) / 255.,
    ]
}

#[cfg(test)]
mod tests {
    use crate::{ColorSpace, OpaqueColor, Srgb};

    fn assert_temperature(kelvin: f32, linear_rgb: [f32; 3]) {
        let result = OpaqueColor::<Srgb>::from_kelvin(kelvin).to_rgba8();
        let expected = OpaqueColor::<Srgb>::new(Srgb::from_linear_srgb(linear_rgb)).to_rgba8();
        assert_eq!(result, expected,
            "Failed getting color from temperature `{kelvin}`. Expected: `{expected}`. Got: `{result}`.");
    }

    #[test]
    fn rgb_from_kelvin() {
        // These expected values are in linear RGB
        assert_temperature(2700., [1., 0.6538038, 0.3427666]);
        assert_temperature(6600., [1., 1., 1.]);
    }
}
