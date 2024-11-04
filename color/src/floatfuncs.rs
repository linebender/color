// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Shims for math functions that ordinarily come from std.

/// Defines a trait that chooses between libstd or libm implementations of float methods.
macro_rules! define_float_funcs {
    ($(
        fn $name:ident(self $(,$arg:ident: $arg_ty:ty)*) -> $ret:ty
        => $lname:ident/$lfname:ident;
    )+) => {

        /// Since core doesn't depend upon libm, this provides libm implementations
        /// of float functions which are typically provided by the std library, when
        /// the `std` feature is not enabled.
        ///
        /// For documentation see the respective functions in the std library.
        #[cfg(not(feature = "std"))]
        pub(crate) trait FloatFuncs : Sized {
            $(fn $name(self $(,$arg: $arg_ty)*) -> $ret;)+
        }

        #[cfg(not(feature = "std"))]
        impl FloatFuncs for f32 {
            $(fn $name(self $(,$arg: $arg_ty)*) -> $ret {
                #[cfg(feature = "libm")]
                return libm::$lfname(self $(,$arg)*);

                #[cfg(not(feature = "libm"))]
                compile_error!("color requires either the `std` or `libm` feature")
            })+
        }

    }
}

define_float_funcs! {
    fn abs(self) -> Self => fabs/fabsf;
    fn atan2(self, other: Self) -> Self => atan2/atan2f;
    fn cbrt(self) -> Self => cbrt/cbrtf;
    fn ceil(self) -> Self => ceil/ceilf;
    fn copysign(self, sign: Self) -> Self => copysign/copysignf;
    fn floor(self) -> Self => floor/floorf;
    fn hypot(self, other: Self) -> Self => hypot/hypotf;
    // Note: powi is missing because its libm implementation is not efficient
    fn powf(self, n: Self) -> Self => pow/powf;
    fn round(self) -> Self => round/roundf;
    fn sin_cos(self) -> (Self, Self) => sincos/sincosf;
    fn sqrt(self) -> Self => sqrt/sqrtf;
    fn trunc(self) -> Self => trunc/truncf;
}
