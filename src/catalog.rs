use crate::foundation::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProductStatus {
    Draft,
    Active,
    Archived,
    Discontinued,
}

domain_struct! {
    pub struct Brand {
        id: Id,
        name: String,
    }
}

domain_struct! {
    pub struct Category {
        id: Id,
        name: String,
    }
}

domain_struct! {
    pub struct Product {
        id: ProductId,
        brand: Brand,
        category: Category,
        status: ProductStatus,
    }
}

domain_struct! {
    pub struct ProductVariant {
        id: VariantId,
        product_id: ProductId,
        sku: Sku,
        active: bool,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProductCatalogEntry {
    pub(crate) product: Product,
    pub(crate) variant: ProductVariant,
}

impl ProductCatalogEntry {
    pub fn try_new(product: Product, variant: ProductVariant) -> DomainResult<Self> {
        if variant.product_id != product.id {
            return Err(ValidationError::Invariant("variant must belong to product"));
        }
        Ok(Self { product, variant })
    }

    pub fn product(&self) -> &Product {
        &self.product
    }

    pub fn variant(&self) -> &ProductVariant {
        &self.variant
    }
}

domain_struct! {
    pub struct ImageAsset {
        id: Id,
        width: Nat,
        height: Nat,
    }
}

domain_struct! {
    pub struct ListingContent {
        title_length: Nat,
        image_count: Nat,
        required_attributes_filled: bool,
    }
}

domain_struct! {
    pub struct MarketplaceContentPolicy {
        max_title_length: Nat,
        min_image_count: Nat,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidListingContent {
    pub(crate) content: ListingContent,
    pub(crate) policy: MarketplaceContentPolicy,
}

impl ValidListingContent {
    pub fn try_new(
        content: ListingContent,
        policy: MarketplaceContentPolicy,
    ) -> DomainResult<Self> {
        if content.title_length > policy.max_title_length {
            return Err(ValidationError::Invariant("listing title exceeds policy"));
        }
        if content.image_count < policy.min_image_count {
            return Err(ValidationError::Invariant("listing has too few images"));
        }
        if !content.required_attributes_filled {
            return Err(ValidationError::Invariant(
                "listing attributes must be filled",
            ));
        }
        Ok(Self { content, policy })
    }

    pub fn content(&self) -> &ListingContent {
        &self.content
    }

    pub fn policy(&self) -> &MarketplaceContentPolicy {
        &self.policy
    }
}
