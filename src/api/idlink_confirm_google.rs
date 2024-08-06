use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct IdLinkConfirmGoogle {
  #[serde(with = "crate::bool_as_int")]
  pub islink: bool,
}

impl CallCustom for IdLinkConfirmGoogle {}
