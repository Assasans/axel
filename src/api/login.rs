use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct Login {
  pub user_no: String,
  pub user_key: String,
  pub user_name: String,
  pub tutorial: u8,
  pub created_at: String,
}

impl CallCustom for Login {}
