// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
// LINEBENDER LINT SET - v1
// See https://linebender.org/wiki/canonical-lints/
// These lints aren't included in Cargo.toml because they
// shouldn't apply to examples and tests
#![warn(unused_crate_dependencies)]
#![warn(clippy::print_stdout, clippy::print_stderr)]

//! # Color
