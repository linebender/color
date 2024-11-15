// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(unsafe_code, reason = "unsafe is required for bytemuck unsafe impls")]

use crate::{AlphaColor, ColorSpace, OpaqueColor, PremulColor, Rgba8};

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
unsafe impl bytemuck::Pod for Rgba8 {}

// Safety: The struct is `repr(C)` and all members are bytemuck::Zeroable.
unsafe impl bytemuck::Zeroable for Rgba8 {}

#[cfg(test)]
mod tests {
    use crate::{AlphaColor, OpaqueColor, PremulColor, Rgba8, Srgb};
    use bytemuck::{TransparentWrapper, Zeroable};
    use core::marker::PhantomData;

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
    }
}
