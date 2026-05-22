//! Executable validators for converting raw boundary data into safe records.

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawCartLine {
    pub sku: Sku,
    pub price: Money,
    pub cost: Money,
    pub quantity: Quantity,
    pub discount: Money,
    pub weight: Weight,
}

pub fn validate_cart_line(raw: RawCartLine) -> Result<CartLine, ValidationError> {
    CartLine::try_new(
        raw.sku,
        raw.price,
        raw.cost,
        raw.quantity,
        raw.discount,
        raw.weight,
    )
    .map_err(|_| ValidationError::LineDiscountExceedsGross)
}

#[must_use]
pub fn cart_line_matches_raw(raw: &RawCartLine, line: &CartLine) -> bool {
    line.sku() == raw.sku
        && line.price() == raw.price
        && line.cost() == raw.cost
        && line.quantity() == raw.quantity
        && line.discount() == raw.discount
        && line.weight() == raw.weight
}

pub fn validate_cart_lines(raw: Vec<RawCartLine>) -> Result<Vec<CartLine>, ValidationError> {
    raw.into_iter().map(validate_cart_line).collect()
}

#[must_use]
pub fn cart_lines_match_raw(raw: &[RawCartLine], lines: &[CartLine]) -> bool {
    raw.len() == lines.len()
        && raw
            .iter()
            .zip(lines)
            .all(|(raw, line)| cart_line_matches_raw(raw, line))
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawOrder {
    pub id: OrderId,
    pub items: Vec<RawCartLine>,
    pub coupon_amount: Money,
    pub shipping_method: ShippingMethod,
    pub tax: Money,
    pub currency: Currency,
    pub status: OrderStatus,
    pub total: Money,
}

pub fn validate_order(raw: RawOrder) -> Result<Order, ValidationError> {
    let items = validate_cart_lines(raw.items)?;
    if raw.coupon_amount > cart_net_total(&items)? {
        return Err(ValidationError::CouponExceedsSubtotal);
    }
    if !shipping_available(&raw.shipping_method, cart_weight_total(&items)?) {
        return Err(ValidationError::ShippingUnavailable);
    }
    let expected = order_total(&raw.shipping_method, raw.coupon_amount, raw.tax, &items)?;
    if raw.total != expected {
        return Err(ValidationError::OrderTotalMismatch);
    }
    Order::try_new(
        raw.id,
        items,
        raw.coupon_amount,
        raw.shipping_method,
        raw.tax,
        raw.currency,
        raw.status,
        raw.total,
    )
}

#[must_use]
pub fn order_matches_raw(raw: &RawOrder, order: &Order) -> bool {
    order.id() == raw.id
        && cart_lines_match_raw(&raw.items, order.items())
        && order.coupon_amount() == raw.coupon_amount
        && order.shipping_method() == &raw.shipping_method
        && order.tax() == raw.tax
        && order.currency() == raw.currency
        && order.status() == raw.status
        && order.total() == raw.total
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawStockState {
    pub sku: Sku,
    pub total: Quantity,
    pub reserved: Quantity,
}

pub fn validate_stock_state(raw: RawStockState) -> Result<StockState, ValidationError> {
    StockState::try_new(raw.sku, raw.total, raw.reserved)
        .map_err(|_| ValidationError::StockReservedExceedsTotal)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawChannelPricePolicy {
    pub min_price: Money,
    pub max_price: Money,
}

pub fn validate_channel_price_policy(
    raw: RawChannelPricePolicy,
) -> Result<ChannelPricePolicy, ValidationError> {
    ChannelPricePolicy::try_new(raw.min_price, raw.max_price)
        .map_err(|_| ValidationError::PricePolicyInvalid)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawProductFeedLine {
    pub sku: Sku,
    pub channel: SalesChannel,
    pub price: Money,
    pub currency: Currency,
    pub stock: Quantity,
    pub stock_state: RawStockState,
    pub price_policy: RawChannelPricePolicy,
}

pub fn validate_feed_line(raw: RawProductFeedLine) -> Result<SafeProductFeedLine, ValidationError> {
    let stock_state = validate_stock_state(raw.stock_state)?;
    let price_policy = validate_channel_price_policy(raw.price_policy)?;
    if raw.sku != stock_state.sku() {
        return Err(ValidationError::FeedSkuMismatch);
    }
    if !valid_channel_price(&price_policy, raw.price) {
        return Err(ValidationError::FeedPriceOutOfPolicy);
    }
    if raw.stock > available_stock(&stock_state) {
        return Err(ValidationError::FeedStockUnavailable);
    }
    SafeProductFeedLine::try_new(
        raw.sku,
        raw.channel,
        raw.price,
        raw.currency,
        raw.stock,
        stock_state,
        price_policy,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawPaymentLedger {
    pub captured: Money,
    pub refunded: Money,
}

pub fn validate_payment_ledger(raw: RawPaymentLedger) -> Result<PaymentLedger, ValidationError> {
    PaymentLedger::try_new(raw.captured, raw.refunded)
        .map_err(|_| ValidationError::LedgerRefundedExceedsCaptured)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawRefund {
    pub amount: Money,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidRefund {
    pub(crate) ledger: PaymentLedger,
    pub(crate) amount: Money,
}

impl ValidRefund {
    #[must_use]
    pub const fn ledger(&self) -> &PaymentLedger {
        &self.ledger
    }

    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }
}

pub fn validate_refund(
    raw: RawRefund,
    ledger: PaymentLedger,
) -> Result<ValidRefund, ValidationError> {
    if !can_refund(&ledger, raw.amount) {
        return Err(ValidationError::RefundExceedsRemaining);
    }
    Ok(ValidRefund {
        ledger,
        amount: raw.amount,
    })
}

pub fn issue_valid_refund(refund: &ValidRefund) -> DomainResult<PaymentLedger> {
    issue_refund(&refund.ledger, refund.amount)
}

pub fn validate_basis_points(value: Nat) -> Result<BasisPoints, ValidationError> {
    BasisPoints::try_new(value).map_err(|_| ValidationError::BasisPointsOutOfRange)
}

pub fn validate_product_catalog_entry(
    product: Product,
    variant: ProductVariant,
) -> Result<ProductCatalogEntry, ValidationError> {
    ProductCatalogEntry::try_new(product, variant)
        .map_err(|_| ValidationError::CatalogInvariantFailed)
}

pub fn validate_listing_content(
    content: ListingContent,
    policy: MarketplaceContentPolicy,
) -> Result<ValidListingContent, ValidationError> {
    ValidListingContent::try_new(content, policy)
        .map_err(|_| ValidationError::CatalogInvariantFailed)
}

pub fn validate_versioned_stock(
    raw: RawStockState,
    version: Nat,
) -> Result<VersionedStock, ValidationError> {
    let stock = validate_stock_state(raw)?;
    Ok(VersionedStock::from_stock(stock, version))
}

pub fn validate_pick_task(
    sku: Sku,
    requested: Quantity,
    bin: BinStock,
) -> Result<PickTask, ValidationError> {
    PickTask::try_new(sku, requested, bin).map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_pack_task(
    source_quantity: Quantity,
    packed_quantity: Quantity,
) -> Result<PackTask, ValidationError> {
    PackTask::try_new(source_quantity, packed_quantity)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_warehouse_shipment(
    packed: Quantity,
    shipped: Quantity,
) -> Result<WarehouseShipment, ValidationError> {
    WarehouseShipment::try_new(packed, shipped)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_allocation(
    node: InventoryNode,
    quantity: Quantity,
) -> Result<Allocation, ValidationError> {
    Allocation::try_new(node, quantity).map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_fulfillment_plan(
    requested: Quantity,
    allocations: Vec<Allocation>,
) -> Result<FulfillmentPlan, ValidationError> {
    FulfillmentPlan::try_new(requested, allocations)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_distinct_fulfillment_plan(
    requested: Quantity,
    allocations: Vec<Allocation>,
) -> Result<DistinctFulfillmentPlan, ValidationError> {
    DistinctFulfillmentPlan::try_new(requested, allocations)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawReservationAttempt {
    pub stock: RawStockState,
    pub version: Nat,
    pub quantity: Quantity,
    pub expected_version: Nat,
}

pub fn validate_raw_reservation_attempt(
    raw: RawReservationAttempt,
) -> Result<ReservationAttempt, ValidationError> {
    Ok(ReservationAttempt::new(
        validate_versioned_stock(raw.stock, raw.version)?,
        raw.quantity,
        raw.expected_version,
    ))
}

pub fn validate_compare_and_swap_reservation(
    stock: VersionedStock,
    quantity: Quantity,
    expected_version: Nat,
) -> Result<VersionedStock, ValidationError> {
    compare_and_swap_reserve(&stock, quantity, expected_version)
        .ok_or(ValidationError::InventoryInvariantFailed)
}

pub fn validate_raw_compare_and_swap_reservation(
    raw: RawReservationAttempt,
) -> Result<VersionedStock, ValidationError> {
    let attempt = validate_raw_reservation_attempt(raw)?;
    validate_compare_and_swap_reservation(
        attempt.stock(),
        attempt.quantity(),
        attempt.expected_version(),
    )
}

pub fn validate_release_reserved_stock(
    stock: StockState,
    quantity: Quantity,
) -> Result<StockState, ValidationError> {
    release_reserved_stock(&stock, quantity).map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_confirm_reserved_shipment(
    stock: StockState,
    quantity: Quantity,
) -> Result<StockState, ValidationError> {
    confirm_reserved_shipment(&stock, quantity)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_timed_reservation(
    stock: StockState,
    quantity: Quantity,
    reserved_at: Timestamp,
    expires_at: Timestamp,
    status: ReservationStatus,
) -> Result<TimedReservation, ValidationError> {
    TimedReservation::try_new(stock, quantity, reserved_at, expires_at, status)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_release_expired_reservation(
    reservation: TimedReservation,
    now: Timestamp,
) -> Result<StockState, ValidationError> {
    release_expired_reservation(&reservation, now)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_backorder_request(
    sku: Sku,
    requested: Quantity,
    available_now: Quantity,
    backordered: Quantity,
) -> Result<BackorderRequest, ValidationError> {
    BackorderRequest::try_new(sku, requested, available_now, backordered)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_preorder_window(
    sku: Sku,
    opens_at: Timestamp,
    closes_at: Timestamp,
    capacity: Quantity,
) -> Result<PreorderWindow, ValidationError> {
    PreorderWindow::try_new(sku, opens_at, closes_at, capacity)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_preorder_reservation(
    window: PreorderWindow,
    quantity: Quantity,
    reserved_at: Timestamp,
) -> Result<PreorderReservation, ValidationError> {
    PreorderReservation::try_new(window, quantity, reserved_at)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_serialized_inventory_set(
    units: Vec<SerializedInventoryUnit>,
) -> Result<SerializedInventorySet, ValidationError> {
    SerializedInventorySet::try_new(units).map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_usable_inventory_lot(
    lot: InventoryLot,
    now: Timestamp,
) -> Result<InventoryLot, ValidationError> {
    if lot_usable_at(now, &lot) {
        Ok(lot)
    } else {
        Err(ValidationError::InventoryInvariantFailed)
    }
}

pub fn validate_sku_substitution(
    requested_sku: Sku,
    substitute_sku: Sku,
    substitute_stock: StockState,
    max_substitute_qty: Quantity,
) -> Result<SkuSubstitution, ValidationError> {
    SkuSubstitution::try_new(
        requested_sku,
        substitute_sku,
        substitute_stock,
        max_substitute_qty,
    )
    .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub fn validate_split_fulfillment_plan(
    plan: DistinctFulfillmentPlan,
    first_warehouse: Warehouse,
    second_warehouse: Warehouse,
) -> Result<SplitFulfillmentPlan, ValidationError> {
    SplitFulfillmentPlan::try_new(plan, first_warehouse, second_warehouse)
        .map_err(|_| ValidationError::InventoryInvariantFailed)
}

pub const fn validate_typed_order<S: OrderStatusMarker>(
    id: OrderId,
    total: Money,
    currency: Currency,
) -> Result<TypedOrder<S>, ValidationError> {
    TypedOrder::try_new(id, total, currency)
}

pub const fn validate_typed_payment<S: PaymentStateMarker>(
    id: PaymentId,
    order_id: OrderId,
    amount: Money,
    currency: Currency,
) -> Result<TypedPayment<S>, ValidationError> {
    TypedPayment::try_new(id, order_id, amount, currency)
}

pub fn validate_balanced_journal_entry(
    postings: Vec<Posting>,
) -> Result<BalancedJournalEntry, ValidationError> {
    BalancedJournalEntry::try_new(postings).map_err(|_| ValidationError::AccountingInvariantFailed)
}

pub fn validate_synced_marketplace_listing(
    listing: MarketplaceListing,
    stock: StockState,
) -> Result<SyncedMarketplaceListing, ValidationError> {
    SyncedMarketplaceListing::try_new(listing, stock)
        .map_err(|_| ValidationError::MarketplaceInvariantFailed)
}

pub fn validate_marketplace_fee_ledger(
    gross: Money,
    fee_rate: BasisPoints,
    fee_rounding_mode: RoundingMode,
    fee: Money,
    payout: Money,
) -> Result<MarketplaceFeeLedger, ValidationError> {
    MarketplaceFeeLedger::try_new(gross, fee_rate, fee_rounding_mode, fee, payout)
        .map_err(|_| ValidationError::MarketplaceInvariantFailed)
}

pub fn validate_marketplace_payout_calculation(
    gross: Money,
    payout_rate: BasisPoints,
    payout_rounding_mode: RoundingMode,
    payout: Money,
) -> Result<MarketplacePayoutCalculation, ValidationError> {
    MarketplacePayoutCalculation::try_new(gross, payout_rate, payout_rounding_mode, payout)
        .map_err(|_| ValidationError::MarketplaceInvariantFailed)
}

pub fn validate_marketplace_order(
    marketplace: Marketplace,
    external_order_id: MarketplaceOrderId,
    internal_order: Order,
    gross_from_marketplace: Money,
    fee_ledger: MarketplaceFeeLedger,
) -> Result<MarketplaceOrder, ValidationError> {
    MarketplaceOrder::try_new(
        marketplace,
        external_order_id,
        internal_order,
        gross_from_marketplace,
        fee_ledger,
    )
    .map_err(|_| ValidationError::MarketplaceInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_marketing_campaign(
    id: CampaignId,
    platform: AdPlatform,
    ad_type: AdType,
    destination: AdDestination,
    status: CampaignStatus,
    budget: Money,
    spend: Money,
    impressions: Nat,
    clicks: Nat,
    conversions: Nat,
    attributed_revenue: Money,
) -> Result<MarketingCampaign, ValidationError> {
    MarketingCampaign::try_new(
        id,
        platform,
        ad_type,
        destination,
        status,
        budget,
        spend,
        impressions,
        clicks,
        conversions,
        attributed_revenue,
    )
    .map_err(|_| ValidationError::MarketingInvariantFailed)
}

pub fn validate_click_attributed_campaign(
    campaign: MarketingCampaign,
) -> Result<ClickAttributedCampaign, ValidationError> {
    ClickAttributedCampaign::try_new(campaign)
        .map_err(|_| ValidationError::MarketingInvariantFailed)
}

pub fn validate_funnel(
    visitors: Nat,
    add_to_cart: Nat,
    checkout_started: Nat,
    purchases: Nat,
) -> Result<Funnel, ValidationError> {
    Funnel::try_new(visitors, add_to_cart, checkout_started, purchases)
        .map_err(|_| ValidationError::MarketingInvariantFailed)
}

pub fn validate_order_attribution_ledger(
    order: Order,
    credits: Vec<AttributionCredit>,
) -> Result<OrderAttributionLedger, ValidationError> {
    OrderAttributionLedger::try_new(order, credits)
        .map_err(|_| ValidationError::MarketingInvariantFailed)
}

pub fn validate_experiment_variant(
    id: Id,
    traffic_weight: Nat,
    visitors: Nat,
    conversions: Nat,
) -> Result<ExperimentVariant, ValidationError> {
    ExperimentVariant::try_new(id, traffic_weight, visitors, conversions)
        .map_err(|_| ValidationError::MarketingInvariantFailed)
}

pub fn validate_experiment(
    id: Id,
    variants: Vec<ExperimentVariant>,
) -> Result<Experiment, ValidationError> {
    Experiment::try_new(id, variants).map_err(|_| ValidationError::MarketingInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_trade_price_book_entry(
    sku: Sku,
    currency: Currency,
    unit_cost: Money,
    retail_unit_price: Money,
    wholesale_unit_price: Money,
    retail_margin: Money,
    wholesale_margin: Money,
    wholesale_min_qty: Quantity,
) -> Result<TradePriceBookEntry, ValidationError> {
    TradePriceBookEntry::try_new(
        sku,
        currency,
        unit_cost,
        retail_unit_price,
        wholesale_unit_price,
        retail_margin,
        wholesale_margin,
        wholesale_min_qty,
    )
    .map_err(|_| ValidationError::B2BInvariantFailed)
}

pub fn validate_retail_line(
    entry: TradePriceBookEntry,
    quantity: Quantity,
    discount: Money,
) -> Result<RetailLine, ValidationError> {
    RetailLine::try_new(entry, quantity, discount).map_err(|_| ValidationError::B2BInvariantFailed)
}

pub fn validate_wholesale_line(
    entry: TradePriceBookEntry,
    quantity: Quantity,
    discount: Money,
) -> Result<WholesaleLine, ValidationError> {
    WholesaleLine::try_new(entry, quantity, discount)
        .map_err(|_| ValidationError::B2BInvariantFailed)
}

pub fn validate_wholesale_credit_account(
    customer: Customer,
    credit_limit: Money,
    outstanding: Money,
) -> Result<WholesaleCreditAccount, ValidationError> {
    WholesaleCreditAccount::try_new(customer, credit_limit, outstanding)
        .map_err(|_| ValidationError::B2BInvariantFailed)
}

pub fn validate_supplier_daily_capacity(
    supplier: DropshipSupplier,
    daily_order_capacity: Nat,
    accepted_orders: Nat,
) -> Result<SupplierDailyCapacity, ValidationError> {
    SupplierDailyCapacity::try_new(supplier, daily_order_capacity, accepted_orders)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_dropship_offer(
    sku: Sku,
    supplier: DropshipSupplier,
    supplier_unit_cost: Money,
    sale_unit_price: Money,
    supplier_shipping_per_unit: Money,
    available_qty: Quantity,
    currency: Currency,
    active: bool,
) -> Result<DropshipOffer, ValidationError> {
    DropshipOffer::try_new(
        sku,
        supplier,
        supplier_unit_cost,
        sale_unit_price,
        supplier_shipping_per_unit,
        available_qty,
        currency,
        active,
    )
    .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_supplier_reservation(
    offer: DropshipOffer,
    supplier: DropshipSupplier,
    quantity: Quantity,
    status: SupplierReservationStatus,
) -> Result<SupplierReservation, ValidationError> {
    SupplierReservation::try_new(offer, supplier, quantity, status)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_dropship_line(
    offer: DropshipOffer,
    quantity: Quantity,
    discount: Money,
) -> Result<DropshipLine, ValidationError> {
    DropshipLine::try_new(offer, quantity, discount)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_reserved_dropship_line(
    line: DropshipLine,
    reservation: SupplierReservation,
) -> Result<ReservedDropshipLine, ValidationError> {
    ReservedDropshipLine::try_new(line, reservation)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_dropship_purchase_order(
    supplier: DropshipSupplier,
    lines: Vec<DropshipLine>,
    quote: DropshipShippingQuote,
    status: DropshipPOStatus,
    total_supplier_cost: Money,
) -> Result<DropshipPurchaseOrder, ValidationError> {
    DropshipPurchaseOrder::try_new(supplier, lines, quote, status, total_supplier_cost)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_dropship_fulfillment(
    customer_order: Order,
    purchase_order: DropshipPurchaseOrder,
    segment_revenue: Money,
) -> Result<DropshipFulfillment, ValidationError> {
    DropshipFulfillment::try_new(customer_order, purchase_order, segment_revenue)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_dropship_return_request(
    line: DropshipLine,
    return_qty: Quantity,
    customer_refund: Money,
    supplier_credit: Money,
) -> Result<DropshipReturnRequest, ValidationError> {
    DropshipReturnRequest::try_new(line, return_qty, customer_refund, supplier_credit)
        .map_err(|_| ValidationError::DropshippingInvariantFailed)
}

pub fn validate_guaranteed_dropship_profit_quote(
    revenue: Money,
    costs: DropshipProfitCosts,
    min_profit: Money,
    profit: Money,
    signed_profit: SignedMoney,
) -> Result<GuaranteedDropshipProfitQuote, ValidationError> {
    GuaranteedDropshipProfitQuote::try_new(revenue, costs, min_profit, profit, signed_profit)
        .map_err(|_| ValidationError::ProfitInvariantFailed)
}

pub fn validate_dropship_cost_upper_bounds(
    actual: DropshipProfitCosts,
    upper: DropshipProfitCosts,
) -> Result<DropshipCostUpperBounds, ValidationError> {
    DropshipCostUpperBounds::try_new(actual, upper)
        .map_err(|_| ValidationError::ProfitInvariantFailed)
}

pub fn validate_singleton_competitor_price_benchmark(
    sku: Sku,
    currency: Currency,
    offer: CompetitorOffer,
) -> Result<CompetitorPriceBenchmark, ValidationError> {
    CompetitorPriceBenchmark::try_new(sku, currency, vec![offer.clone()], offer)
        .map_err(|_| ValidationError::CompetitorInvariantFailed)
}

pub fn validate_competitor_aware_dropship_offer(
    offer: DropshipOffer,
    benchmark: CompetitorPriceBenchmark,
    discount: Money,
    costs: DropshipProfitCosts,
    min_profit: Money,
) -> Result<CompetitorAwareDropshipOffer, ValidationError> {
    CompetitorAwareDropshipOffer::try_new(offer, benchmark, discount, costs, min_profit)
        .map_err(|_| ValidationError::CompetitorInvariantFailed)
}

pub fn validate_brand_pricing_policy(
    map_price: Money,
    msrp: Money,
) -> Result<BrandPricingPolicy, ValidationError> {
    BrandPricingPolicy::try_new(map_price, msrp)
        .map_err(|_| ValidationError::MerchandisingInvariantFailed)
}

pub fn validate_bundle_component(
    sku: Sku,
    units_per_bundle: Quantity,
    stock_available: Quantity,
) -> Result<BundleComponent, ValidationError> {
    BundleComponent::try_new(sku, units_per_bundle, stock_available)
        .map_err(|_| ValidationError::MerchandisingInvariantFailed)
}

pub fn bundle_components_can_fulfill_all(
    bundle_qty: Quantity,
    components: &[BundleComponent],
) -> DomainResult<bool> {
    for component in components {
        if !component_can_fulfill_bundles(bundle_qty, component)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn validate_bundle_reservation(
    bundle_qty: Quantity,
    components: Vec<BundleComponent>,
) -> Result<BundleReservation, ValidationError> {
    if !bundle_components_can_fulfill_all(bundle_qty, &components)
        .map_err(|_| ValidationError::MerchandisingInvariantFailed)?
    {
        return Err(ValidationError::MerchandisingInvariantFailed);
    }
    BundleReservation::try_new(bundle_qty, components)
        .map_err(|_| ValidationError::MerchandisingInvariantFailed)
}

pub fn validate_accepted_promotion_set(
    resulting_price: Money,
    total_discount: Money,
    discount_cap: Money,
    profit_floor: Money,
) -> Result<AcceptedPromotionSet, ValidationError> {
    AcceptedPromotionSet::try_new(resulting_price, total_discount, discount_cap, profit_floor)
        .map_err(|_| ValidationError::MerchandisingInvariantFailed)
}

pub fn validate_search_result_item(
    item: SearchResultItem,
) -> Result<ValidSearchResultItem, ValidationError> {
    ValidSearchResultItem::try_new(item).map_err(|_| ValidationError::MerchandisingInvariantFailed)
}

pub fn validate_exchange_rate(
    source: Currency,
    target: Currency,
    numerator: Nat,
    denominator: Nat,
    observed_at: Timestamp,
) -> Result<ExchangeRate, ValidationError> {
    ExchangeRate::try_new(source, target, numerator, denominator, observed_at)
        .map_err(|_| ValidationError::FinanceInvariantFailed)
}

pub fn validate_tax_calculation(
    taxable_amount: Money,
    rate: TaxRate,
    rounding_mode: RoundingMode,
    tax: Money,
    total: Money,
) -> Result<TaxCalculation, ValidationError> {
    TaxCalculation::try_new(taxable_amount, rate, rounding_mode, tax, total)
        .map_err(|_| ValidationError::FinanceInvariantFailed)
}

pub fn validate_tax_inclusive_price(
    gross: Money,
    net: Money,
    tax: Money,
) -> Result<TaxInclusivePrice, ValidationError> {
    TaxInclusivePrice::try_new(gross, net, tax).map_err(|_| ValidationError::TaxInvariantFailed)
}

pub fn validate_tax_exclusive_price(
    net: Money,
    tax: Money,
    total: Money,
) -> Result<TaxExclusivePrice, ValidationError> {
    TaxExclusivePrice::try_new(net, tax, total).map_err(|_| ValidationError::TaxInvariantFailed)
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawTaxInvoiceLine {
    pub sku: Sku,
    pub quantity: Quantity,
    pub unit_price: Money,
    pub discount: Money,
    pub treatment: TaxTreatment,
    pub rate: TaxRate,
    pub rounding_mode: RoundingMode,
    pub taxable_amount: Money,
    pub tax: Money,
    pub total: Money,
}

pub fn validate_tax_invoice_line(
    raw: RawTaxInvoiceLine,
) -> Result<TaxInvoiceLine, ValidationError> {
    TaxInvoiceLine::try_new(
        raw.sku,
        raw.quantity,
        raw.unit_price,
        raw.discount,
        raw.treatment,
        raw.rate,
        raw.rounding_mode,
        raw.taxable_amount,
        raw.tax,
        raw.total,
    )
    .map_err(|_| ValidationError::TaxInvariantFailed)
}

pub fn validate_tax_invoice_lines(
    raw: Vec<RawTaxInvoiceLine>,
) -> Result<Vec<TaxInvoiceLine>, ValidationError> {
    raw.into_iter().map(validate_tax_invoice_line).collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawTaxInvoice {
    pub id: Id,
    pub issued_at: Timestamp,
    pub seller_id: Id,
    pub buyer_id: CustomerId,
    pub jurisdiction: TaxJurisdiction,
    pub currency: Currency,
    pub lines: Vec<RawTaxInvoiceLine>,
    pub subtotal: Money,
    pub tax: Money,
    pub shipping: Money,
    pub discount: Money,
    pub total: Money,
}

pub fn validate_tax_invoice(raw: RawTaxInvoice) -> Result<TaxInvoice, ValidationError> {
    let lines = validate_tax_invoice_lines(raw.lines)?;
    TaxInvoice::try_new(
        raw.id,
        raw.issued_at,
        raw.seller_id,
        raw.buyer_id,
        raw.jurisdiction,
        raw.currency,
        lines,
        raw.subtotal,
        raw.tax,
        raw.shipping,
        raw.discount,
        raw.total,
    )
    .map_err(|_| ValidationError::TaxInvariantFailed)
}

pub fn validate_order_tax_invoice_link(
    order: Order,
    invoice: TaxInvoice,
) -> Result<OrderTaxInvoiceLink, ValidationError> {
    OrderTaxInvoiceLink::try_new(order, invoice).map_err(|_| ValidationError::TaxInvariantFailed)
}

pub fn validate_tax_exemption_certificate(
    customer_id: CustomerId,
    jurisdiction_id: Id,
    valid_from: Timestamp,
    valid_until: Timestamp,
) -> Result<TaxExemptionCertificate, ValidationError> {
    TaxExemptionCertificate::try_new(customer_id, jurisdiction_id, valid_from, valid_until)
        .map_err(|_| ValidationError::TaxInvariantFailed)
}

pub fn validate_b2b_tax_exemption(
    customer: Customer,
    jurisdiction: TaxJurisdiction,
    certificate: TaxExemptionCertificate,
    checked_at: Timestamp,
) -> Result<B2BTaxExemption, ValidationError> {
    B2BTaxExemption::try_new(customer, jurisdiction, certificate, checked_at)
        .map_err(|_| ValidationError::TaxInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_marketplace_facilitator_tax(
    marketplace: Marketplace,
    jurisdiction: TaxJurisdiction,
    taxable_amount: Money,
    rate: TaxRate,
    rounding_mode: RoundingMode,
    tax: Money,
    facilitator_collects: bool,
    seller_tax_due: Money,
) -> Result<MarketplaceFacilitatorTax, ValidationError> {
    MarketplaceFacilitatorTax::try_new(
        marketplace,
        jurisdiction,
        taxable_amount,
        rate,
        rounding_mode,
        tax,
        facilitator_collects,
        seller_tax_due,
    )
    .map_err(|_| ValidationError::TaxInvariantFailed)
}

pub fn validate_carrier_quote(
    service: CarrierService,
    package: Package,
    price: Money,
) -> Result<CarrierQuote, ValidationError> {
    CarrierQuote::try_new(service, package, price)
        .map_err(|_| ValidationError::FinanceInvariantFailed)
}

pub fn validate_reconciliation_within_tolerance(
    expected: Money,
    actual: Money,
    tolerance: Money,
) -> Result<ReconciliationWithinTolerance, ValidationError> {
    ReconciliationWithinTolerance::try_new(expected, actual, tolerance)
        .map_err(|_| ValidationError::FinanceInvariantFailed)
}

pub fn validate_subscription_plan(
    price: Money,
    period_days: Days,
) -> Result<SubscriptionPlan, ValidationError> {
    SubscriptionPlan::try_new(price, period_days)
        .map_err(|_| ValidationError::PostPurchaseInvariantFailed)
}

pub fn validate_recurring_subscription(
    customer: CustomerId,
    plan: SubscriptionPlan,
    status: SubscriptionLifecycleStatus,
    current_billing_date: Timestamp,
    next_billing_date: Timestamp,
) -> Result<RecurringSubscription, ValidationError> {
    RecurringSubscription::try_new(
        customer,
        plan,
        status,
        current_billing_date,
        next_billing_date,
    )
    .map_err(|_| ValidationError::PostPurchaseInvariantFailed)
}

pub fn validate_gift_card_redemption(
    card: GiftCard,
    amount: Money,
) -> Result<GiftCardRedemption, ValidationError> {
    GiftCardRedemption::try_new(card, amount)
        .map_err(|_| ValidationError::PostPurchaseInvariantFailed)
}

pub fn validate_chargeback(
    payment_amount: Money,
    chargeback_amount: Money,
) -> Result<Chargeback, ValidationError> {
    Chargeback::try_new(payment_amount, chargeback_amount)
        .map_err(|_| ValidationError::PostPurchaseInvariantFailed)
}

pub fn validate_cashflow_plan(
    starting_cash: Money,
    required_reserve: Money,
    expected_inflows: Money,
    expected_outflows: Money,
) -> Result<CashflowPlan, ValidationError> {
    CashflowPlan::try_new(
        starting_cash,
        required_reserve,
        expected_inflows,
        expected_outflows,
    )
    .map_err(|_| ValidationError::PostPurchaseInvariantFailed)
}

pub fn validate_event_backed_cashflow_plan(
    starting_cash: Money,
    required_reserve: Money,
    events: Vec<CashflowEvent>,
) -> Result<EventBackedCashflowPlan, ValidationError> {
    EventBackedCashflowPlan::try_new(starting_cash, required_reserve, events)
        .map_err(|_| ValidationError::PostPurchaseInvariantFailed)
}

pub fn validate_audited_command(
    actor: Role,
    action: Action,
    order_id: OrderId,
    event: AuditEvent,
) -> Result<AuditedCommand, ValidationError> {
    AuditedCommand::try_new(actor, action, order_id, event)
        .map_err(|_| ValidationError::AuditPermissionDenied)
}

pub fn validate_audited_entity_command(
    actor: Role,
    action: Action,
    subject_id: Id,
    event: EntityAuditEvent,
) -> Result<AuditedEntityCommand, ValidationError> {
    AuditedEntityCommand::try_new(actor, action, subject_id, event)
        .map_err(|_| ValidationError::AuditPermissionDenied)
}

pub fn validate_event_stream(stream: EventStream) -> Result<ValidEventStream, ValidationError> {
    ValidEventStream::try_new(stream).map_err(|_| ValidationError::EventStreamInvalid)
}

pub fn validate_approved_supplier_quality(
    supplier: DropshipSupplier,
    metrics: SupplierQualityMetrics,
    policy: SupplierRiskPolicy,
) -> Result<ApprovedSupplierQuality, ValidationError> {
    ApprovedSupplierQuality::try_new(supplier, metrics, policy)
        .map_err(|_| ValidationError::SupplierQualityInvalid)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_dropship_opportunity_candidate(
    sku: Sku,
    units: Quantity,
    target_price: Money,
    required_capital: Money,
    expected_profit: Money,
    min_profit: Money,
    competitor_price: Money,
    costs: DropshipProfitCosts,
) -> Result<DropshipOpportunityCandidate, ValidationError> {
    DropshipOpportunityCandidate::try_new(
        sku,
        units,
        target_price,
        required_capital,
        expected_profit,
        min_profit,
        competitor_price,
        costs,
    )
    .map_err(|_| ValidationError::OpportunityInvariantFailed)
}

pub fn validate_dropship_opportunity_portfolio(
    selected: Vec<DropshipOpportunityCandidate>,
    investment_fund: Money,
) -> Result<DropshipOpportunityPortfolio, ValidationError> {
    DropshipOpportunityPortfolio::try_new(selected, investment_fund)
        .map_err(|_| ValidationError::OpportunityInvariantFailed)
}

pub fn validate_crm_account(
    id: AccountId,
    customer: Customer,
    tier: AccountTier,
    status: CRMAccountStatus,
    lifetime_value: Money,
    open_balance: Money,
) -> Result<CRMAccount, ValidationError> {
    CRMAccount::try_new(id, customer, tier, status, lifetime_value, open_balance)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_active_crm_account(
    account: CRMAccount,
) -> Result<ActiveCRMAccount, ValidationError> {
    ActiveCRMAccount::try_new(account).map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_crm_account_contact(
    account: CRMAccount,
    contact: CRMContact,
) -> Result<CRMAccountContact, ValidationError> {
    CRMAccountContact::try_new(account, contact).map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_permitted_customer_message(
    interaction_id: InteractionId,
    contact: CRMContact,
    sent_at: Timestamp,
) -> Result<PermittedCustomerMessage, ValidationError> {
    PermittedCustomerMessage::try_new(interaction_id, contact, sent_at)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_permitted_account_message(
    account_contact: CRMAccountContact,
    message: PermittedCustomerMessage,
) -> Result<PermittedAccountMessage, ValidationError> {
    PermittedAccountMessage::try_new(account_contact, message)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_crm_interaction(
    id: InteractionId,
    account_id: AccountId,
    contact_id: ContactId,
    kind: InteractionKind,
    occurred_at: Timestamp,
    follow_up_due_at: Timestamp,
) -> Result<CRMInteraction, ValidationError> {
    CRMInteraction::try_new(
        id,
        account_id,
        contact_id,
        kind,
        occurred_at,
        follow_up_due_at,
    )
    .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_crm_interaction_for_contact(
    account_contact: CRMAccountContact,
    interaction: CRMInteraction,
) -> Result<CRMInteractionForContact, ValidationError> {
    CRMInteractionForContact::try_new(account_contact, interaction)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_lead(
    id: LeadId,
    account_id: AccountId,
    contact_id: ContactId,
    source_campaign: Option<CampaignId>,
    status: LeadStatus,
    estimated_value: Money,
    currency: Currency,
    created_at: Timestamp,
    updated_at: Timestamp,
) -> Result<Lead, ValidationError> {
    Lead::try_new(
        id,
        account_id,
        contact_id,
        source_campaign,
        status,
        estimated_value,
        currency,
        created_at,
        updated_at,
    )
    .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_lead_for_contact(
    account_contact: CRMAccountContact,
    lead: Lead,
) -> Result<LeadForContact, ValidationError> {
    LeadForContact::try_new(account_contact, lead).map_err(|_| ValidationError::CrmInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_sales_opportunity(
    id: OpportunityId,
    account_id: AccountId,
    contact_id: ContactId,
    source_lead: Option<LeadId>,
    stage: OpportunityStage,
    amount: Money,
    currency: Currency,
    probability: BasisPoints,
    opened_at: Timestamp,
    updated_at: Timestamp,
    expected_close_at: Timestamp,
) -> Result<SalesOpportunity, ValidationError> {
    SalesOpportunity::try_new(
        id,
        account_id,
        contact_id,
        source_lead,
        stage,
        amount,
        currency,
        probability,
        opened_at,
        updated_at,
        expected_close_at,
    )
    .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_opportunity_for_contact(
    account_contact: CRMAccountContact,
    opportunity: SalesOpportunity,
) -> Result<OpportunityForContact, ValidationError> {
    OpportunityForContact::try_new(account_contact, opportunity)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_sales_pipeline(
    currency: Currency,
    opportunities: Vec<SalesOpportunity>,
) -> Result<SalesPipeline, ValidationError> {
    SalesPipeline::try_new(currency, opportunities).map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_customer_segment(
    id: SegmentId,
    name: String,
    member_count: Nat,
    min_lifetime_value: Money,
    max_retention_discount: Money,
) -> Result<CustomerSegment, ValidationError> {
    CustomerSegment::try_new(
        id,
        name,
        member_count,
        min_lifetime_value,
        max_retention_discount,
    )
    .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_segment_membership(
    account: CRMAccount,
    segment: CustomerSegment,
) -> Result<SegmentMembership, ValidationError> {
    SegmentMembership::try_new(account, segment).map_err(|_| ValidationError::CrmInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_support_case(
    id: SupportCaseId,
    account_id: AccountId,
    contact_id: ContactId,
    order_id: Option<OrderId>,
    status: SupportCaseStatus,
    priority: SupportPriority,
    opened_at: Timestamp,
    last_updated_at: Timestamp,
    sla_due_at: Timestamp,
) -> Result<SupportCase, ValidationError> {
    SupportCase::try_new(
        id,
        account_id,
        contact_id,
        order_id,
        status,
        priority,
        opened_at,
        last_updated_at,
        sla_due_at,
    )
    .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_support_case_for_contact(
    account_contact: CRMAccountContact,
    case_: SupportCase,
) -> Result<SupportCaseForContact, ValidationError> {
    SupportCaseForContact::try_new(account_contact, case_)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_resolved_support_case(
    case_: SupportCase,
    resolved_at: Timestamp,
) -> Result<ResolvedSupportCase, ValidationError> {
    ResolvedSupportCase::try_new(case_, resolved_at)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

pub fn validate_retention_offer(
    account: CRMAccount,
    segment: CustomerSegment,
    coupon: Coupon,
    uses_before: Nat,
    discount: Money,
) -> Result<RetentionOffer, ValidationError> {
    RetentionOffer::try_new(account, segment, coupon, uses_before, discount)
        .map_err(|_| ValidationError::CrmInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_logistics_shipment_plan(
    id: ShipmentId,
    order: Order,
    fulfillment: DistinctFulfillmentPlan,
    quote: CarrierQuote,
    warehouse: Warehouse,
    destination: ShippingDestination,
    planned_ship_at: Timestamp,
    promised_delivery_at: Timestamp,
) -> Result<LogisticsShipmentPlan, ValidationError> {
    let package = quote.package().clone();
    LogisticsShipmentPlan::try_new(
        id,
        order,
        fulfillment,
        package,
        quote,
        warehouse,
        destination,
        planned_ship_at,
        promised_delivery_at,
    )
    .map_err(|_| ValidationError::LogisticsInvariantFailed)
}

pub fn validate_logistics_shipment(
    id: ShipmentId,
    plan: LogisticsShipmentPlan,
    status: ShipmentStatus,
    created_at: Timestamp,
    updated_at: Timestamp,
) -> Result<LogisticsShipment, ValidationError> {
    LogisticsShipment::try_new(id, plan, status, created_at, updated_at)
        .map_err(|_| ValidationError::LogisticsInvariantFailed)
}

pub fn validate_carrier_handoff(
    plan: LogisticsShipmentPlan,
    tracking_number: Id,
    handed_off_at: Timestamp,
    acceptance_scan_at: Timestamp,
) -> Result<CarrierHandoff, ValidationError> {
    let service = plan.quote().service().clone();
    CarrierHandoff::try_new(
        plan,
        service,
        tracking_number,
        handed_off_at,
        acceptance_scan_at,
    )
    .map_err(|_| ValidationError::LogisticsInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_warehouse_transfer(
    id: TransferId,
    sku: Sku,
    from_warehouse: Warehouse,
    to_warehouse: Warehouse,
    source_stock: StockState,
    requested: Quantity,
    in_transit: Quantity,
    received: Quantity,
) -> Result<WarehouseTransfer, ValidationError> {
    WarehouseTransfer::try_new(
        id,
        sku,
        from_warehouse,
        to_warehouse,
        source_stock,
        requested,
        in_transit,
        received,
    )
    .map_err(|_| ValidationError::LogisticsInvariantFailed)
}

#[allow(clippy::too_many_arguments)]
pub fn validate_return_authorization(
    id: ReturnAuthorizationId,
    support_case: SupportCase,
    order: Order,
    ledger: PaymentLedger,
    status: ReturnAuthorizationStatus,
    lines: Vec<ReturnLine>,
    quantity: Quantity,
    refund_amount: Money,
    requested_at: Timestamp,
    decided_at: Timestamp,
) -> Result<ReturnAuthorization, ValidationError> {
    ReturnAuthorization::try_new(
        id,
        support_case,
        order,
        ledger,
        status,
        lines,
        quantity,
        refund_amount,
        requested_at,
        decided_at,
    )
    .map_err(|_| ValidationError::LogisticsInvariantFailed)
}

pub fn validate_return_receipt(
    authorization: ReturnAuthorization,
    received_quantity: Quantity,
    refund_issued: Money,
    received_at: Timestamp,
) -> Result<ReturnReceipt, ValidationError> {
    ReturnReceipt::try_new(authorization, received_quantity, refund_issued, received_at)
        .map_err(|_| ValidationError::LogisticsInvariantFailed)
}

pub fn validate_bounded_coupon_application(
    coupon: Coupon,
    subtotal: Money,
    uses_before: Nat,
) -> Result<BoundedCouponApplication, ValidationError> {
    BoundedCouponApplication::try_new(coupon, subtotal, uses_before)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_captured_payment_matches_order(
    order: Order,
    payment: CapturedPayment,
) -> Result<CapturedPaymentMatchesOrder, ValidationError> {
    CapturedPaymentMatchesOrder::try_new(order, payment)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_sellable_catalog_entry(
    entry: ProductCatalogEntry,
) -> Result<SellableCatalogEntry, ValidationError> {
    SellableCatalogEntry::try_new(entry).map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_publishable_feed_line(
    line: SafeProductFeedLine,
) -> Result<PublishableFeedLine, ValidationError> {
    PublishableFeedLine::try_new(line).map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_sourceable_distributor_product(
    product: DistributorProduct,
    units: Quantity,
) -> Result<SourceableDistributorProduct, ValidationError> {
    SourceableDistributorProduct::try_new(product, units)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_fraud_checked_coupon_application(
    application: BoundedCouponApplication,
    policy: FraudPolicy,
) -> Result<FraudCheckedCouponApplication, ValidationError> {
    FraudCheckedCouponApplication::try_new(application, policy)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_captured_payment_journal_projection(
    accounts: AccountingAccounts,
    payment: CapturedPayment,
) -> Result<CapturedPaymentJournalProjection, ValidationError> {
    let journal = payment_captured_journal(&accounts, payment.amount())
        .map_err(|_| ValidationError::ImplicitInvariantFailed)?;
    CapturedPaymentJournalProjection::try_new(accounts, payment, journal)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_refund_journal_projection(
    accounts: AccountingAccounts,
    ledger: PaymentLedger,
    amount: Money,
) -> Result<RefundJournalProjection, ValidationError> {
    let journal = refund_issued_journal(&accounts, amount)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)?;
    RefundJournalProjection::try_new(accounts, ledger, amount, journal)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_advertisable_synced_marketplace_listing(
    synced: SyncedMarketplaceListing,
) -> Result<AdvertisableSyncedMarketplaceListing, ValidationError> {
    AdvertisableSyncedMarketplaceListing::try_new(synced)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_wholesale_credit_checkout(
    account: WholesaleCreditAccount,
    lines: Vec<WholesaleLine>,
    terms: PaymentTerms,
    order_total: Money,
) -> Result<WholesaleCreditCheckout, ValidationError> {
    WholesaleCreditCheckout::try_new(account, lines, terms, order_total)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_trusted_fresh_competitor_benchmark(
    benchmark: CompetitorPriceBenchmark,
    now: Timestamp,
    max_age: Duration,
    trust: TrustLevel,
) -> Result<TrustedFreshCompetitorBenchmark, ValidationError> {
    TrustedFreshCompetitorBenchmark::try_new(benchmark, now, max_age, trust)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_map_compliant_competitor_aware_offer(
    offer: CompetitorAwareDropshipOffer,
    policy: BrandPricingPolicy,
) -> Result<MapCompliantCompetitorAwareOffer, ValidationError> {
    MapCompliantCompetitorAwareOffer::try_new(offer, policy)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_fresh_currency_conversion(
    source_amount: MoneyAmount,
    rate: ExchangeRate,
    target_amount: MoneyAmount,
    now: Timestamp,
    max_age: Duration,
) -> Result<FreshCurrencyConversion, ValidationError> {
    FreshCurrencyConversion::try_new(source_amount, rate, target_amount, now, max_age)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_valid_gift_card_redemption_at(
    now: Timestamp,
    redemption: GiftCardRedemption,
) -> Result<ValidGiftCardRedemptionAt, ValidationError> {
    ValidGiftCardRedemptionAt::try_new(now, redemption)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_chargeback_for_captured_payment(
    payment: CapturedPayment,
    chargeback: Chargeback,
) -> Result<ChargebackForCapturedPayment, ValidationError> {
    ChargebackForCapturedPayment::try_new(payment, chargeback)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_actionable_demand_forecast(
    forecast: DemandForecast,
) -> Result<ActionableDemandForecast, ValidationError> {
    ActionableDemandForecast::try_new(forecast)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_approved_orderable_supplier_quality(
    quality: ApprovedSupplierQuality,
) -> Result<ApprovedOrderableSupplierQuality, ValidationError> {
    ApprovedOrderableSupplierQuality::try_new(quality)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_converted_lead_opportunity(
    lead: Lead,
    opportunity: SalesOpportunity,
) -> Result<ConvertedLeadOpportunity, ValidationError> {
    ConvertedLeadOpportunity::try_new(lead, opportunity)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_crm_order_contact(
    account: CRMAccount,
    contact: CRMContact,
    order: Order,
) -> Result<CRMOrderContact, ValidationError> {
    CRMOrderContact::try_new(account, contact, order)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_shipment_for_crm_order(
    crm_order: CRMOrderContact,
    plan: LogisticsShipmentPlan,
) -> Result<ShipmentForCRMOrder, ValidationError> {
    ShipmentForCRMOrder::try_new(crm_order, plan)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_logistics_exception_support_case(
    exception: LogisticsException,
    shipment: LogisticsShipmentPlan,
    support_case: SupportCase,
) -> Result<LogisticsExceptionSupportCase, ValidationError> {
    LogisticsExceptionSupportCase::try_new(exception, shipment, support_case)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}

pub fn validate_crm_approved_return_handling(
    authorization: ReturnAuthorization,
    receipt: ReturnReceipt,
) -> Result<CRMApprovedReturnHandling, ValidationError> {
    CRMApprovedReturnHandling::try_new(authorization, receipt)
        .map_err(|_| ValidationError::ImplicitInvariantFailed)
}
