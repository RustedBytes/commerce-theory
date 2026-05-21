# commerce-theory

Runtime Rust mirror of the `CommerceTheory` Lean package.

This crate turns the Lean model's proof-carrying commerce records into ordinary
Rust domain types with private fields, smart constructors, executable
predicates, checked arithmetic, and tests for the headline safety guarantees.

## What It Models

The crate covers e-commerce and marketplace invariants across:

- foundation units, IDs, currency, money, basis points, and profit arithmetic
- catalog, inventory, pricing, orders, payments, refunds, and accounting
- marketplace listings, feeds, payouts, marketing, B2B, and wholesale credit
- dropshipping, supplier capacity, purchase orders, returns, and profit floors
- competitor pricing, merchandising, fulfillment finance, risk, and privacy
- CRM accounts, contacts, leads, opportunities, segments, and support cases
- logistics shipment plans, tracking, warehouse transfers, and returns
- tax treatments, invoices, exemptions, and marketplace facilitator tax
- raw input validation helpers that compose the smart constructors
- event sourcing, event validation, replay, workflows, keyed totals, and ranking

All modules are re-exported from the library root:

```rust
use commerce_theory::*;
```

## Design

Lean proof fields are represented as runtime validation:

- data fields are private;
- `try_new` constructors enforce invariants;
- simple records also provide `new` where there is no invariant to check;
- predicates return `bool`;
- arithmetic that can overflow returns `Result<_, ValidationError>`;
- natural-number subtraction uses saturating subtraction to match Lean `Nat`
  subtraction flooring at zero.

The crate uses `u128` aliases for Lean `Nat`-style quantities and `time` types
for timestamps and durations:

```rust
pub type MinorUnit = u128;
pub type NonNegMoney = MinorUnit;
pub type Money = NonNegMoney;
pub type SignedMoney = i128;
pub type Quantity = u128;
pub type Weight = u128;
pub type Timestamp = time::PrimitiveDateTime;
pub type Date = time::Date;
pub type Duration = time::Duration;
pub type Days = Duration;
pub type Id = u128;
```

## Example

```rust
use commerce_theory::*;

fn main() -> Result<(), ValidationError> {
    let line = CartLine::try_new(Sku::new(1), 100, 40, 2, 20, 3)?;
    assert_eq!(line_gross_total(&line)?, 200);
    assert_eq!(line_net_total(&line)?, 180);

    let shipping = ShippingMethod::new(15, 500, 20);
    let items = vec![line];
    let total = order_total(&shipping, 10, 5, &items)?;

    let order = Order::try_new(
        OrderId::new(7),
        items,
        10,
        shipping,
        5,
        Currency::USD,
        OrderStatus::New,
        total,
    )?;

    assert_eq!(order.total(), total);
    Ok(())
}
```

## Optional Serde Support

Serde derives are available behind the `serde` feature:

```bash
cargo test --features serde
```

## Verification

Run the Rust checks:

```bash
cargo fmt --check
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --all-features --no-deps
```

## Crate Layout

- `src/foundation.rs`: shared IDs, units, money, currencies, validation errors
- `src/catalog.rs`, `src/inventory.rs`, `src/pricing.rs`, `src/orders.rs`
- `src/accounting.rs`, `src/marketplace.rs`, `src/marketing.rs`, `src/b2b.rs`
- `src/dropshipping.rs`, `src/dropship_profit.rs`, `src/competitor_pricing.rs`
- `src/merchandising.rs`, `src/fulfillment_finance.rs`, `src/risk_privacy.rs`
- `src/basic.rs`, `src/crm.rs`, `src/logistics.rs`, `src/tax.rs`,
  `src/validation.rs`
- `src/event_sourcing.rs`, `src/event_language.rs`, `src/event_replay.rs`
- `src/post_purchase.rs`, `src/forecasting.rs`, `src/implicit_invariants.rs`
- `src/inventory_algorithms.rs`, `src/keyed_totals.rs`,
  `src/opportunity_portfolio.rs`, `src/opportunity_ranking.rs`,
  `src/workflow.rs`
- `src/summary.rs`: theorem-style regression tests for the runtime mirror
