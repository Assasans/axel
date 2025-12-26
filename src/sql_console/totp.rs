use std::time::{SystemTime, UNIX_EPOCH};
use constant_time_eq::constant_time_eq;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_totp(secret: &[u8], time_step: u64) -> String {
  // Get current time and calculate counter (60-second interval)
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs();
  let counter = now / time_step;

  // Convert counter to 8 bytes big-endian
  let counter_bytes = counter.to_be_bytes();

  // HMAC-SHA256
  let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
  mac.update(&counter_bytes);
  let result = mac.finalize().into_bytes();

  // Dynamic truncation
  let offset = (result[result.len() - 1] & 0x0f) as usize;
  let binary = ((result[offset] & 0x7f) as u64) << 24
    | (result[offset + 1] as u64) << 16
    | (result[offset + 2] as u64) << 8
    | (result[offset + 3] as u64);

  // 9-digit code
  let otp = binary % 1_000_000_000;
  format!("{:09}", otp)
}

pub fn verify_totp(secret: &[u8], provided_otp: &str, time_step: u64) -> bool {
  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs();

  // Check previous, current, and next time window for clock skew tolerance
  for offset in [-1i64, 0, 1] {
    let counter = ((now as i64) / (time_step as i64) + offset) as u64;
    let counter_bytes = counter.to_be_bytes();

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();

    // Dynamic truncation
    let dyn_offset = (result[result.len() - 1] & 0x0f) as usize;
    let binary = ((result[dyn_offset] & 0x7f) as u64) << 24
      | (result[dyn_offset + 1] as u64) << 16
      | (result[dyn_offset + 2] as u64) << 8
      | (result[dyn_offset + 3] as u64);

    let otp = binary % 1_000_000_000;
    let expected = format!("{:09}", otp);

    if constant_time_eq(expected.as_bytes(), provided_otp.as_bytes()) {
      return true;
    }
  }
  false
}
