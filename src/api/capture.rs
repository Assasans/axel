use std::io::{Cursor, Read};
use std::sync::Arc;

use base64::prelude::{BASE64_STANDARD, BASE64_STANDARD_NO_PAD};
use base64::Engine;
use serde::Deserialize;
use tracing::debug;

use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Signed};
use crate::user::session::Session;

#[derive(Debug, Deserialize)]
pub struct CaptureRequest {
  pub version: String,
  #[serde(rename = "serializedData")]
  pub serialized_data: String,
}

#[derive(Debug, Deserialize)]
pub struct CaptureDeserialized {
  #[serde(rename = "tutoriaDataJson")]
  pub tutoria_data_json: String,
  #[serde(rename = "userLocalSettingsJson")]
  pub user_local_settings_json: String,
}

#[derive(Debug, Deserialize)]
pub struct CaptureSendRequest {
  pub capture: String,
}

// Well... capture = base64(json(base64(gzip(json(json(TutorialData) + json(UserLocalSettings))))))
// It seems to be a telemetry endpoint without API side effects.
pub async fn capture_send(
  session: Arc<Session>,
  Params(params): Params<CaptureSendRequest>,
) -> impl IntoHandlerResponse {
  let capture = BASE64_STANDARD_NO_PAD
    .decode(params.capture)
    .expect("failed to decode capture from base64");
  let capture = serde_json::from_slice::<CaptureRequest>(&capture).expect("failed to deserialize capture");

  let capture_data = BASE64_STANDARD
    .decode(capture.serialized_data)
    .expect("failed to decode serialized data from base64");
  let mut decoder = flate2::read::GzDecoder::new(Cursor::new(capture_data));
  let mut deserialized_data = String::new();
  decoder
    .read_to_string(&mut deserialized_data)
    .expect("failed to read gzipped data");
  let deserialized_data: CaptureDeserialized =
    serde_json::from_str(&deserialized_data).expect("failed to deserialize capture data");

  debug!("tutorial data: {}", deserialized_data.tutoria_data_json);
  debug!("user local settings: {}", deserialized_data.user_local_settings_json);

  Signed((), session)
}
