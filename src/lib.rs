//! Runtime Rust mirror of the `CommerceTheory` Lean package.
//!
//! The Lean package stores proof fields in validated records. This crate mirrors
//! those records with private fields, smart constructors, executable predicates,
//! and tests that exercise the same safety guarantees at runtime.

#![forbid(unsafe_code)]
#![allow(clippy::too_many_arguments)]

macro_rules! domain_struct {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $vis struct $name {
            $(pub(crate) $field: $ty),*
        }

        impl $name {
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }

            pub fn try_new($($field: $ty),*) -> Result<Self, $crate::foundation::ValidationError> {
                Ok(Self::new($($field),*))
            }

            $(
                pub fn $field(&self) -> &$ty {
                    &self.$field
                }
            )*
        }
    };
}

macro_rules! impl_getters {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        impl $name {
            $(
                pub fn $field(&self) -> &$ty {
                    &self.$field
                }
            )*
        }
    };
}

pub mod accounting;
pub mod b2b;
pub mod catalog;
pub mod competitor_pricing;
pub mod dropship_profit;
pub mod dropshipping;
pub mod event_language;
pub mod event_replay;
pub mod event_sourcing;
pub mod forecasting;
pub mod foundation;
pub mod fulfillment_finance;
pub mod implicit_invariants;
pub mod inventory;
pub mod inventory_algorithms;
pub mod keyed_totals;
pub mod marketing;
pub mod marketplace;
pub mod merchandising;
pub mod opportunity_portfolio;
pub mod opportunity_ranking;
pub mod orders;
pub mod post_purchase;
pub mod pricing;
pub mod risk_privacy;
pub mod summary;
pub mod workflow;

pub use accounting::*;
pub use b2b::*;
pub use catalog::*;
pub use competitor_pricing::*;
pub use dropship_profit::*;
pub use dropshipping::*;
pub use event_language::*;
pub use event_replay::*;
pub use event_sourcing::*;
pub use forecasting::*;
pub use foundation::*;
pub use fulfillment_finance::*;
pub use implicit_invariants::*;
pub use inventory::*;
pub use inventory_algorithms::*;
pub use keyed_totals::*;
pub use marketing::*;
pub use marketplace::*;
pub use merchandising::*;
pub use opportunity_portfolio::*;
pub use opportunity_ranking::*;
pub use orders::*;
pub use post_purchase::*;
pub use pricing::*;
pub use risk_privacy::*;
pub use workflow::*;
