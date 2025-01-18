// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

// In Rust 1.84 (https://github.com/rust-lang/rust/pull/131304), `abs` and
// `copysign` were added to `core`, so we no longer need these forwarded to
// libm.
#![cfg_attr(
    not(feature = "std"),
    allow(dead_code, reason = "abs and copysign were added to core in 1.84")
)]

//! Shims for math functions that ordinarily come from std.

/// Defines a trait that chooses between libstd or libm implementations of float methods.
macro_rules! define_float_funcs {
    ($(
        fn $name:ident(self $(,$arg:ident: $arg_ty:ty)*) -> $ret:ty
        => $lfname:ident;
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
    // This is not needed once the MSRV is 1.84 or later.
    fn abs(self) -> Self => fabsf;
    fn atan2(self, other: Self) -> Self => atan2f;
    fn cbrt(self) -> Self => cbrtf;
    fn ceil(self) -> Self => ceilf;
    // This is not needed once the MSRV is 1.84 or later.
    fn copysign(self, sign: Self) -> Self => copysignf;
    fn floor(self) -> Self => floorf;
    fn hypot(self, other: Self) -> Self => hypotf;
    // Note: powi is missing because its libm implementation is not efficient
    fn powf(self, n: Self) -> Self => powf;
    fn round(self) -> Self => roundf;
    fn sin_cos(self) -> (Self, Self) => sincosf;
    fn sqrt(self) -> Self => sqrtf;
}
