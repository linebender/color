// Copyright 2026 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[cfg(all(not(feature = "std"), not(test)))]
use crate::floatfuncs::FloatFuncs;

/// A per-channel component transfer function.
///
/// The table, discrete, linear, and gamma variants correspond to the SVG and Filter Effects
/// `feComponentTransfer` function types. These functions are evaluated on straight, not
/// premultiplied, components.
///
/// See [Filter Effects Module Level 1 § 8.6][fe-component-transfer].
///
/// [fe-component-transfer]: https://www.w3.org/TR/filter-effects-1/#feComponentTransferElement
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransferFunction<'a> {
    /// Return the input component unchanged.
    Identity,
    /// Linearly interpolate between table values over the input range `[0, 1]`.
    ///
    /// An empty table is treated as identity. A one-entry table maps every input to that entry.
    /// Inputs outside `[0, 1]` use the nearest endpoint.
    Table(&'a [f32]),
    /// Select a table value as a step function over the input range `[0, 1]`.
    ///
    /// An empty table is treated as identity. Inputs outside `[0, 1]` use the nearest endpoint.
    Discrete(&'a [f32]),
    /// Apply `slope * component + intercept`.
    Linear {
        /// The multiplication factor.
        slope: f32,
        /// The value added after multiplication.
        intercept: f32,
    },
    /// Apply `amplitude * component.powf(exponent) + offset`.
    Gamma {
        /// The multiplication factor applied after exponentiation.
        amplitude: f32,
        /// The exponent passed to `powf`.
        exponent: f32,
        /// The value added after exponentiation and multiplication.
        offset: f32,
    },
}

impl<'a> TransferFunction<'a> {
    /// The identity transfer function.
    pub const IDENTITY: Self = Self::Identity;

    /// Create a linear transfer function.
    #[inline]
    #[must_use]
    pub const fn linear(slope: f32, intercept: f32) -> Self {
        Self::Linear { slope, intercept }
    }

    /// Create a gamma transfer function.
    #[inline]
    #[must_use]
    pub const fn gamma(amplitude: f32, exponent: f32, offset: f32) -> Self {
        Self::Gamma {
            amplitude,
            exponent,
            offset,
        }
    }

    /// Create a table transfer function.
    #[inline]
    #[must_use]
    pub const fn table(values: &'a [f32]) -> Self {
        Self::Table(values)
    }

    /// Create a discrete transfer function.
    #[inline]
    #[must_use]
    pub const fn discrete(values: &'a [f32]) -> Self {
        Self::Discrete(values)
    }

    /// Apply this transfer function to a straight component.
    #[inline]
    #[must_use]
    pub fn apply(self, component: f32) -> f32 {
        match self {
            Self::Identity => component,
            Self::Table(values) => apply_table(values, component),
            Self::Discrete(values) => apply_discrete(values, component),
            Self::Linear { slope, intercept } => slope * component + intercept,
            Self::Gamma {
                amplitude,
                exponent,
                offset,
            } => amplitude * component.powf(exponent) + offset,
        }
    }
}

#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "bounded table coordinates are converted to lookup indices"
)]
fn apply_table(values: &[f32], component: f32) -> f32 {
    match values {
        [] => component,
        [value] => *value,
        _ => {
            if component <= 0.0 {
                return values[0];
            }

            let last = values.len() - 1;
            if component >= 1.0 {
                return values[last];
            }

            let scaled = component * last as f32;
            let lower = (scaled as usize).min(last - 1);
            let t = scaled - lower as f32;
            values[lower] + t * (values[lower + 1] - values[lower])
        }
    }
}

#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "bounded table coordinates are converted to lookup indices"
)]
fn apply_discrete(values: &[f32], component: f32) -> f32 {
    match values {
        [] => component,
        [value] => *value,
        _ => {
            if component <= 0.0 {
                return values[0];
            }

            let last = values.len() - 1;
            if component >= 1.0 {
                return values[last];
            }

            let ix = (component * values.len() as f32) as usize;
            values[ix.min(last)]
        }
    }
}
