use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};
use serde::{Serialize, Deserialize};
use tracing::warn;

// See [Wonder_Api_PurchaseGoogleLimitedProductsStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PurchaseGoogleLimitedProductsStatus {
  pub product_list: Vec<GoogleLimitedProductStatus>,
}

impl CallCustom for PurchaseGoogleLimitedProductsStatus {}

// See [Wonder_Api_PurchaseLimitedProductsStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct GoogleLimitedProductStatus {
  pub external_id: String,
  pub purchasable_amount: i32,
  pub end_at: String,
}

pub async fn purchase_google_limited_products_status() -> impl IntoHandlerResponse {
  warn!("encountered stub: purchase_google_limited_products_status");

  Unsigned(PurchaseGoogleLimitedProductsStatus { product_list: vec![] })
}

#[derive(Debug, Deserialize)]
pub struct PurchaseGoogleChargeStatusRequest {
  pub product_id: String,
}

// See [Wonder_Api_PurchaseGoogleChargeStatusResponseDto_Fields]
#[derive(Debug, Serialize)]
pub struct PurchaseGoogleChargeStatus {
  pub charge_total: i32,
}

impl CallCustom for PurchaseGoogleChargeStatus {}

pub async fn purchase_google_charge_status(
  Params(params): Params<PurchaseGoogleChargeStatusRequest>,
) -> impl IntoHandlerResponse {
  warn!(?params, "encountered stub: purchase_google_charge_status");

  Unsigned(PurchaseGoogleChargeStatus { charge_total: 2112 })
}
