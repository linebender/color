// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(unsafe_code, reason = "unsafe is required for bytemuck unsafe impls")]

use crate::{
    cache_key::CacheKey, AlphaColor, ColorSpace, ColorSpaceTag, HueDirection, OpaqueColor,
    PremulColor, PremulRgba8, Rgba8,
};

// Safety: The struct is `repr(transparent)` and the data member is bytemuck::Pod.
unsafe impl<CS: ColorSpace> bytemuck::Pod for AlphaColor<CS> {}

// Safety: The struct is `repr(transparent)`.
unsafe impl<CS: ColorSpace> bytemuck::TransparentWrapper<[f32; 4]> for AlphaColor<CS> {}

// Safety: The struct is `repr(transparent)` and the data member is bytemuck::Zeroable.
unsafe impl<CS: ColorSpace> bytemuck::Zeroable for AlphaColor<CS> {}

// Safety: The struct is `repr(transparent)` and the data member is bytemuck::Pod.
unsafe impl<CS: ColorSpace> bytemuck::Pod for OpaqueColor<CS> {}

// Safety: The struct is `repr(transparent)`.
unsafe impl<CS: ColorSpace> bytemuck::TransparentWrapper<[f32; 3]> for OpaqueColor<CS> {}

// Safety: The struct is `repr(transparent)` and the data member is bytemuck::Zeroable.
unsafe impl<CS: ColorSpace> bytemuck::Zeroable for OpaqueColor<CS> {}

// Safety: The struct is `repr(transparent)` and the data member is bytemuck::Pod.
unsafe impl<CS: ColorSpace> bytemuck::Pod for PremulColor<CS> {}

// Safety: The struct is `repr(transparent)`.
unsafe impl<CS: ColorSpace> bytemuck::TransparentWrapper<[f32; 4]> for PremulColor<CS> {}

// Safety: The struct is `repr(transparent)` and the data member is bytemuck::Zeroable.
unsafe impl<CS: ColorSpace> bytemuck::Zeroable for PremulColor<CS> {}

// Safety: The struct is `repr(C)` and all members are bytemuck::Pod.
unsafe impl bytemuck::Pod for PremulRgba8 {}

// Safety: The struct is `repr(C)` and all members are bytemuck::Zeroable.
unsafe impl bytemuck::Zeroable for PremulRgba8 {}

// Safety: The struct is `repr(C)` and all members are bytemuck::Pod.
unsafe impl bytemuck::Pod for Rgba8 {}

// Safety: The struct is `repr(C)` and all members are bytemuck::Zeroable.
unsafe impl bytemuck::Zeroable for Rgba8 {}

// Safety: The enum is `repr(u8)` and has only fieldless variants.
unsafe impl bytemuck::NoUninit for ColorSpaceTag {}

// Safety: The enum is `repr(u8)` and `0` is a valid value.
unsafe impl bytemuck::Zeroable for ColorSpaceTag {}

// Safety: The enum is `repr(u8)`.
unsafe impl bytemuck::checked::CheckedBitPattern for ColorSpaceTag {
    type Bits = u8;

    fn is_valid_bit_pattern(bits: &u8) -> bool {
        use bytemuck::Contiguous;
        // Don't need to compare against MIN_VALUE as this is u8 and 0 is the MIN_VALUE.
        *bits <= Self::MAX_VALUE
    }
}

// Safety: The enum is `repr(u8)`. All values are `u8` and fall within
// the min and max values.
unsafe impl bytemuck::Contiguous for ColorSpaceTag {
    type Int = u8;
    const MIN_VALUE: u8 = Self::Srgb as u8;
    const MAX_VALUE: u8 = Self::Aces2065_1 as u8;
}

// Safety: The enum is `repr(u8)` and has only fieldless variants.
unsafe impl bytemuck::NoUninit for HueDirection {}

// Safety: The enum is `repr(u8)` and `0` is a valid value.
unsafe impl bytemuck::Zeroable for HueDirection {}

// Safety: The enum is `repr(u8)`.
unsafe impl bytemuck::checked::CheckedBitPattern for HueDirection {
    type Bits = u8;

    fn is_valid_bit_pattern(bits: &u8) -> bool {
        use bytemuck::Contiguous;
        // Don't need to compare against MIN_VALUE as this is u8 and 0 is the MIN_VALUE.
        *bits <= Self::MAX_VALUE
    }
}

// Safety: The enum is `repr(u8)`. All values are `u8` and fall within
// the min and max values.
unsafe impl bytemuck::Contiguous for HueDirection {
    type Int = u8;
    const MIN_VALUE: u8 = Self::Shorter as u8;
    const MAX_VALUE: u8 = Self::Decreasing as u8;
}

// Safety: The struct is `repr(transparent)`.
unsafe impl<T> bytemuck::TransparentWrapper<T> for CacheKey<T> {}

#[cfg(test)]
mod tests {
    use crate::{
        cache_key::CacheKey, AlphaColor, ColorSpaceTag, HueDirection, OpaqueColor, PremulColor,
        PremulRgba8, Rgba8, Srgb,
    };
    use bytemuck::{checked::try_from_bytes, Contiguous, TransparentWrapper, Zeroable};
    use core::{marker::PhantomData, ptr};

    fn assert_is_pod(_pod: impl bytemuck::Pod) {}

    #[test]
    fn alphacolor_is_pod() {
        let AlphaColor {
            components,
            cs: PhantomData,
        } = AlphaColor::<Srgb>::new([1., 2., 3., 0.]);
        assert_is_pod(components);
    }

    #[test]
    fn opaquecolor_is_pod() {
        let OpaqueColor {
            components,
            cs: PhantomData,
        } = OpaqueColor::<Srgb>::new([1., 2., 3.]);
        assert_is_pod(components);
    }

    #[test]
    fn premulcolor_is_pod() {
        let PremulColor {
            components,
            cs: PhantomData,
        } = PremulColor::<Srgb>::new([1., 2., 3., 0.]);
        assert_is_pod(components);
    }

    #[test]
    fn premulrgba8_is_pod() {
        let rgba8 = PremulRgba8 {
            r: 0,
            b: 0,
            g: 0,
            a: 0,
        };
        let PremulRgba8 { r, g, b, a } = rgba8;
        assert_is_pod(r);
        assert_is_pod(g);
        assert_is_pod(b);
        assert_is_pod(a);
    }

    #[test]
    fn rgba8_is_pod() {
        let rgba8 = Rgba8 {
            r: 0,
            b: 0,
            g: 0,
            a: 0,
        };
        let Rgba8 { r, g, b, a } = rgba8;
        assert_is_pod(r);
        assert_is_pod(g);
        assert_is_pod(b);
        assert_is_pod(a);
    }

    #[test]
    fn checked_bit_pattern() {
        let valid = bytemuck::bytes_of(&2_u8);
        let invalid = bytemuck::bytes_of(&200_u8);

        assert_eq!(
            Ok(&ColorSpaceTag::Lab),
            try_from_bytes::<ColorSpaceTag>(valid)
        );

        assert!(try_from_bytes::<ColorSpaceTag>(invalid).is_err());

        assert_eq!(
            Ok(&HueDirection::Increasing),
            try_from_bytes::<HueDirection>(valid)
        );

        assert!(try_from_bytes::<HueDirection>(invalid).is_err());
    }

    #[test]
    fn contiguous() {
        let cst1 = ColorSpaceTag::LinearSrgb;
        let cst2 = ColorSpaceTag::from_integer(cst1.into_integer());
        assert_eq!(Some(cst1), cst2);

        assert_eq!(None, ColorSpaceTag::from_integer(255));

        let hd1 = HueDirection::Decreasing;
        let hd2 = HueDirection::from_integer(hd1.into_integer());
        assert_eq!(Some(hd1), hd2);

        assert_eq!(None, HueDirection::from_integer(255));
    }

    // If the inner type is wrong in the unsafe impl above,
    // that will result in failures here due to assertions
    // within bytemuck.
    #[test]
    fn transparent_wrapper() {
        let ac = AlphaColor::<Srgb>::new([1., 2., 3., 0.]);
        let ai: [f32; 4] = AlphaColor::<Srgb>::peel(ac);
        assert_eq!(ai, [1., 2., 3., 0.]);

        let oc = OpaqueColor::<Srgb>::new([1., 2., 3.]);
        let oi: [f32; 3] = OpaqueColor::<Srgb>::peel(oc);
        assert_eq!(oi, [1., 2., 3.]);

        let pc = PremulColor::<Srgb>::new([1., 2., 3., 0.]);
        let pi: [f32; 4] = PremulColor::<Srgb>::peel(pc);
        assert_eq!(pi, [1., 2., 3., 0.]);

        let ck = CacheKey::<f32>::new(1.);
        let ci: f32 = CacheKey::<f32>::peel(ck);
        assert_eq!(ci, 1.);
    }

    #[test]
    fn zeroable() {
        let ac = AlphaColor::<Srgb>::zeroed();
        assert_eq!(ac.components, [0., 0., 0., 0.]);

        let oc = OpaqueColor::<Srgb>::zeroed();
        assert_eq!(oc.components, [0., 0., 0.]);

        let pc = PremulColor::<Srgb>::zeroed();
        assert_eq!(pc.components, [0., 0., 0., 0.]);

        let rgba8 = Rgba8::zeroed();
        assert_eq!(
            rgba8,
            Rgba8 {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            }
        );

        let cst = ColorSpaceTag::zeroed();
        assert_eq!(cst, ColorSpaceTag::Srgb);

        let hd = HueDirection::zeroed();
        assert_eq!(hd, HueDirection::Shorter);
    }

    /// Tests that the [`Contiguous`] impl for [`HueDirection`] is not trivially incorrect.
    const _: () = {
        let mut value = 0;
        while value <= HueDirection::MAX_VALUE {
            // Safety: In a const context, therefore if this makes an invalid HueDirection, that will be detected.
            let it: HueDirection = unsafe { ptr::read((&raw const value).cast()) };
            // Evaluate the enum value to ensure it actually has a valid tag
            if it as u8 != value {
                unreachable!();
            }
            value += 1;
        }
    };

    /// Tests that the [`Contiguous`] impl for [`ColorSpaceTag`] is not trivially incorrect.
    const _: () = {
        let mut value = 0;
        while value <= ColorSpaceTag::MAX_VALUE {
            // Safety: In a const context, therefore if this makes an invalid ColorSpaceTag, that will be detected.
            let it: ColorSpaceTag = unsafe { ptr::read((&raw const value).cast()) };
            // Evaluate the enum value to ensure it actually has a valid tag
            if it as u8 != value {
                unreachable!();
            }
            value += 1;
        }
    };
}

#[cfg(doctest)]
/// Doctests aren't collected under `cfg(test)`; we can use `cfg(doctest)` instead
mod doctests {
    /// Validates that any new variants in `HueDirection` has led to a change in the `Contiguous` impl.
    /// Note that to test this robustly, we'd need 256 tests, which is impractical.
    /// We make the assumption that all new variants will maintain contiguousness.
    ///
    /// ```compile_fail,E0080
    /// use bytemuck::Contiguous;
    /// use color::HueDirection;
    /// const {
    ///     let value = HueDirection::MAX_VALUE + 1;
    ///     // Safety: In a const context, therefore if this makes an invalid HueDirection, that will be detected.
    ///     // (Indeed, we rely upon that)
    ///     let it: HueDirection = unsafe { core::ptr::read((&raw const value).cast()) };
    ///     // Evaluate the enum value to ensure it actually has an invalid tag
    ///     if it as u8 != value {
    ///         unreachable!();
    ///     }
    /// }
    /// ```
    const _HUE_DIRECTION: () = {};

    /// Validates that any new variants in `ColorSpaceTag` has led to a change in the `Contiguous` impl.
    /// Note that to test this robustly, we'd need 256 tests, which is impractical.
    /// We make the assumption that all new variants will maintain contiguousness.
    ///
    /// ```compile_fail,E0080
    /// use bytemuck::Contiguous;
    /// use color::ColorSpaceTag;
    /// const {
    ///     let value = ColorSpaceTag::MAX_VALUE + 1;
    ///     let it: ColorSpaceTag = unsafe { core::ptr::read((&raw const value).cast()) };
    ///     // Evaluate the enum value to ensure it actually has an invalid tag
    ///     if it as u8 != value {
    ///         unreachable!();
    ///     }
    /// }
    /// ```
    const _COLOR_SPACE_TAG: () = {};
}
