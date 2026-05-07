// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! # Color Operations
//!
//! Constructors for common color operations.
//!
//! Per-component operations build [`ComponentTransfer`] and [`TransferFunction`] values.
//! Operations that mix color channels build [`ColorMatrix`] values. Mixed pipelines can store
//! those operations as [`ColorOperation`] values. All operations apply to the components of the
//! color space the caller chooses; they do not convert, clip, or gamut-map colors. Common
//! operations are associated constructors, for example [`ComponentTransfer::opacity`] and
//! [`ColorMatrix::grayscale`].
//! Operations can be applied directly to [`color::AlphaColor`], [`color::PremulColor`], and
//! [`color::DynamicColor`].
//! Matrices can be exchanged as row-major 4x5 rows or flattened row-major `[f32; 20]` values.
//!
//! Constructors use arguments as provided. This crate does not apply CSS or SVG shorthand
//! clamping; callers implementing those specifications should clamp at the API boundary.
//!
//! The `std` feature is enabled by default. For `no_std` builds, disable default features and
//! enable the `libm` feature.

// LINEBENDER LINT SET - lib.rs - v4
// See https://linebender.org/wiki/canonical-lints/
// These lints shouldn't apply to examples or tests.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
// These lints shouldn't apply to examples.
#![warn(clippy::print_stdout, clippy::print_stderr)]
// Targeting e.g. 32-bit means structs containing usize can give false positives for 64-bit.
#![cfg_attr(target_pointer_width = "64", warn(clippy::trivially_copy_pass_by_ref))]
// END LINEBENDER LINT SET
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

mod component_transfer;
#[cfg(all(not(feature = "std"), not(test)))]
mod floatfuncs;
mod matrix;
mod operation;
mod target;

pub use component_transfer::ComponentTransfer;
pub use component_transfer::TransferFunction;
pub use matrix::ColorMatrix;
pub use operation::ColorOperation;
pub use target::ColorOperationTarget;

// Keep clippy from complaining about unused libm when `std` and `libm` are both enabled.
#[cfg(feature = "libm")]
#[expect(unused, reason = "keep clippy happy")]
fn ensure_libm_dependency_used() -> f32 {
    libm::sqrtf(4_f32)
}
