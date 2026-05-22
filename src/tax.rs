use crate::b2b::*;
use crate::foundation::*;
use crate::fulfillment_finance::*;
use crate::marketplace::*;
use crate::orders::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaxRegime {
    SalesTax,
    VAT,
    GST,
    Custom,
}

domain_struct! {
    pub struct TaxJurisdiction {
        id: Id,
        name: String,
        regime: TaxRegime,
        currency: Currency,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaxTreatment {
    Taxable,
    Exempt,
    ZeroRated,
    ReverseCharge,
}

#[must_use]
pub fn seller_collects_tax_for_treatment(treatment: TaxTreatment) -> bool {
    treatment == TaxTreatment::Taxable
}

pub fn tax_for_treatment(
    treatment: TaxTreatment,
    mode: RoundingMode,
    rate: &TaxRate,
    taxable_amount: Money,
) -> DomainResult<Money> {
    match treatment {
        TaxTreatment::Taxable => tax_amount_rounded(mode, rate, taxable_amount),
        TaxTreatment::Exempt | TaxTreatment::ZeroRated | TaxTreatment::ReverseCharge => Ok(0),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TaxPriceMode {
    Exclusive,
    Inclusive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaxInclusivePrice {
    pub(crate) gross: Money,
    pub(crate) net: Money,
    pub(crate) tax: Money,
}

impl TaxInclusivePrice {
    pub fn try_new(gross: Money, net: Money, tax: Money) -> DomainResult<Self> {
        if gross != checked_add(net, tax, "tax inclusive price")? {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self { gross, net, tax })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaxExclusivePrice {
    pub(crate) net: Money,
    pub(crate) tax: Money,
    pub(crate) total: Money,
}

impl TaxExclusivePrice {
    pub fn try_new(net: Money, tax: Money, total: Money) -> DomainResult<Self> {
        if total != checked_add(net, tax, "tax exclusive price")? {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self { net, tax, total })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaxInvoiceLine {
    pub(crate) sku: Sku,
    pub(crate) quantity: Quantity,
    pub(crate) unit_price: Money,
    pub(crate) discount: Money,
    pub(crate) treatment: TaxTreatment,
    pub(crate) rate: TaxRate,
    pub(crate) rounding_mode: RoundingMode,
    pub(crate) taxable_amount: Money,
    pub(crate) tax: Money,
    pub(crate) total: Money,
}

impl TaxInvoiceLine {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        sku: Sku,
        quantity: Quantity,
        unit_price: Money,
        discount: Money,
        treatment: TaxTreatment,
        rate: TaxRate,
        rounding_mode: RoundingMode,
        taxable_amount: Money,
        tax: Money,
        total: Money,
    ) -> DomainResult<Self> {
        let gross = checked_mul(unit_price, quantity, "tax invoice line gross")?;
        if discount > gross
            || taxable_amount != nat_sub(gross, discount)
            || tax != tax_for_treatment(treatment, rounding_mode, &rate, taxable_amount)?
            || total != checked_add(taxable_amount, tax, "tax invoice line total")?
        {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self {
            sku,
            quantity,
            unit_price,
            discount,
            treatment,
            rate,
            rounding_mode,
            taxable_amount,
            tax,
            total,
        })
    }
}

pub fn invoice_line_subtotal_total(lines: &[TaxInvoiceLine]) -> DomainResult<Money> {
    checked_sum(
        lines.iter().map(|line| line.taxable_amount),
        "invoice_line_subtotal_total",
    )
}

pub fn invoice_line_tax_total(lines: &[TaxInvoiceLine]) -> DomainResult<Money> {
    checked_sum(lines.iter().map(|line| line.tax), "invoice_line_tax_total")
}

pub fn invoice_line_grand_total(lines: &[TaxInvoiceLine]) -> DomainResult<Money> {
    checked_sum(
        lines.iter().map(|line| line.total),
        "invoice_line_grand_total",
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaxInvoice {
    pub(crate) id: Id,
    pub(crate) issued_at: Timestamp,
    pub(crate) seller_id: Id,
    pub(crate) buyer_id: CustomerId,
    pub(crate) jurisdiction: TaxJurisdiction,
    pub(crate) currency: Currency,
    pub(crate) lines: Vec<TaxInvoiceLine>,
    pub(crate) subtotal: Money,
    pub(crate) tax: Money,
    pub(crate) shipping: Money,
    pub(crate) discount: Money,
    pub(crate) total: Money,
}

impl TaxInvoice {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        id: Id,
        issued_at: Timestamp,
        seller_id: Id,
        buyer_id: CustomerId,
        jurisdiction: TaxJurisdiction,
        currency: Currency,
        lines: Vec<TaxInvoiceLine>,
        subtotal: Money,
        tax: Money,
        shipping: Money,
        discount: Money,
        total: Money,
    ) -> DomainResult<Self> {
        let components = checked_add(
            checked_add(subtotal, tax, "tax invoice subtotal tax")?,
            shipping,
            "tax invoice components",
        )?;
        if subtotal != invoice_line_subtotal_total(&lines)?
            || tax != invoice_line_tax_total(&lines)?
            || discount > components
            || total != nat_sub(components, discount)
        {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self {
            id,
            issued_at,
            seller_id,
            buyer_id,
            jurisdiction,
            currency,
            lines,
            subtotal,
            tax,
            shipping,
            discount,
            total,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderTaxInvoiceLink {
    pub(crate) order: Order,
    pub(crate) invoice: TaxInvoice,
}

impl OrderTaxInvoiceLink {
    pub fn try_new(order: Order, invoice: TaxInvoice) -> DomainResult<Self> {
        if order.tax() != invoice.tax || invoice.currency != order.currency() {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self { order, invoice })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TaxExemptionCertificate {
    pub(crate) customer_id: CustomerId,
    pub(crate) jurisdiction_id: Id,
    pub(crate) valid_from: Timestamp,
    pub(crate) valid_until: Timestamp,
}

impl TaxExemptionCertificate {
    pub fn try_new(
        customer_id: CustomerId,
        jurisdiction_id: Id,
        valid_from: Timestamp,
        valid_until: Timestamp,
    ) -> DomainResult<Self> {
        if valid_until < valid_from {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self {
            customer_id,
            jurisdiction_id,
            valid_from,
            valid_until,
        })
    }
}

#[must_use]
pub fn certificate_valid_at(certificate: &TaxExemptionCertificate, now: Timestamp) -> bool {
    certificate.valid_from <= now && now <= certificate.valid_until
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct B2BTaxExemption {
    pub(crate) customer: Customer,
    pub(crate) jurisdiction: TaxJurisdiction,
    pub(crate) certificate: TaxExemptionCertificate,
    pub(crate) checked_at: Timestamp,
}

impl B2BTaxExemption {
    pub fn try_new(
        customer: Customer,
        jurisdiction: TaxJurisdiction,
        certificate: TaxExemptionCertificate,
        checked_at: Timestamp,
    ) -> DomainResult<Self> {
        if certificate.customer_id != customer.id
            || certificate.jurisdiction_id != jurisdiction.id
            || !customer.wholesale_approved
            || !certificate_valid_at(&certificate, checked_at)
        {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self {
            customer,
            jurisdiction,
            certificate,
            checked_at,
        })
    }
}

#[must_use]
pub const fn seller_tax_due_for_facilitator(facilitator_collects: bool, tax: Money) -> Money {
    if facilitator_collects { 0 } else { tax }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarketplaceFacilitatorTax {
    pub(crate) marketplace: Marketplace,
    pub(crate) jurisdiction: TaxJurisdiction,
    pub(crate) taxable_amount: Money,
    pub(crate) rate: TaxRate,
    pub(crate) rounding_mode: RoundingMode,
    pub(crate) tax: Money,
    pub(crate) facilitator_collects: bool,
    pub(crate) seller_tax_due: Money,
}

impl MarketplaceFacilitatorTax {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        marketplace: Marketplace,
        jurisdiction: TaxJurisdiction,
        taxable_amount: Money,
        rate: TaxRate,
        rounding_mode: RoundingMode,
        tax: Money,
        facilitator_collects: bool,
        seller_tax_due: Money,
    ) -> DomainResult<Self> {
        if tax != tax_amount_rounded(rounding_mode, &rate, taxable_amount)?
            || seller_tax_due != seller_tax_due_for_facilitator(facilitator_collects, tax)
        {
            return Err(ValidationError::TaxInvariantFailed);
        }
        Ok(Self {
            marketplace,
            jurisdiction,
            taxable_amount,
            rate,
            rounding_mode,
            tax,
            facilitator_collects,
            seller_tax_due,
        })
    }
}

pub fn invoice_line_floor_tax_rounding_remainder(line: &TaxInvoiceLine) -> DomainResult<Nat> {
    floor_rounding_remainder(
        checked_mul(
            line.taxable_amount,
            line.rate.bps().value(),
            "invoice line tax remainder",
        )?,
        10_000,
    )
}

pub fn invoice_lines_floor_tax_rounding_remainder_total(
    lines: &[TaxInvoiceLine],
) -> DomainResult<Nat> {
    checked_result_sum(
        lines.iter().map(|line| {
            checked_mul(
                line.taxable_amount,
                line.rate.bps().value(),
                "invoice lines tax remainder",
            )
            .and_then(|numerator| floor_rounding_remainder(numerator, 10_000))
        }),
        "floor_rounded_lines_remainder_total",
    )
}

impl_getters!(TaxInclusivePrice {
    gross: Money,
    net: Money,
    tax: Money,
});

impl_getters!(TaxExclusivePrice {
    net: Money,
    tax: Money,
    total: Money,
});

impl_getters!(TaxInvoiceLine {
    sku: Sku,
    quantity: Quantity,
    unit_price: Money,
    discount: Money,
    treatment: TaxTreatment,
    rate: TaxRate,
    rounding_mode: RoundingMode,
    taxable_amount: Money,
    tax: Money,
    total: Money,
});

impl_getters!(TaxInvoice {
    id: Id,
    issued_at: Timestamp,
    seller_id: Id,
    buyer_id: CustomerId,
    jurisdiction: TaxJurisdiction,
    currency: Currency,
    lines: Vec<TaxInvoiceLine>,
    subtotal: Money,
    tax: Money,
    shipping: Money,
    discount: Money,
    total: Money,
});

impl_getters!(OrderTaxInvoiceLink {
    order: Order,
    invoice: TaxInvoice,
});

impl_getters!(TaxExemptionCertificate {
    customer_id: CustomerId,
    jurisdiction_id: Id,
    valid_from: Timestamp,
    valid_until: Timestamp,
});

impl_getters!(B2BTaxExemption {
    customer: Customer,
    jurisdiction: TaxJurisdiction,
    certificate: TaxExemptionCertificate,
    checked_at: Timestamp,
});

impl_getters!(MarketplaceFacilitatorTax {
    marketplace: Marketplace,
    jurisdiction: TaxJurisdiction,
    taxable_amount: Money,
    rate: TaxRate,
    rounding_mode: RoundingMode,
    tax: Money,
    facilitator_collects: bool,
    seller_tax_due: Money,
});
