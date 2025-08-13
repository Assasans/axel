use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use uuid::Uuid;

/// UUID generation is done in [Wonder.Util.UserData$$SetupNewUser] with the following format
/// (as one string without whitespaces):
///
/// `{System.Guid::NewGuid().ToString("N")}
/// "01g1"
/// {Wonder.Util.UserData::advertisingId}
/// "aabb"
/// {System.Guid::NewGuid().ToString("N")}`
///
/// These two GUIDs always change on logout. `advertisingId` is returned by the Java API.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct UserUuid {
  pub local_uuid_1: Uuid,
  pub local_uuid_2: Uuid,
  pub auth_token: String,
}

impl FromStr for UserUuid {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.len() < 72 {
      return Err("string too short".into());
    }

    let local_uuid_1_str = &s[0..32];
    let marker1 = &s[32..36];
    if marker1 != "01g1" {
      return Err("invalid first marker".into());
    }

    let local_uuid_2_str = &s[s.len() - 32..];
    let marker2_start = s.len() - 36;
    let marker2 = &s[marker2_start..marker2_start + 4];
    if marker2 != "aabb" {
      return Err("invalid second marker".into());
    }

    let auth_token = &s[36..marker2_start];

    let local_uuid_1 = Uuid::from_str(local_uuid_1_str).map_err(|e| format!("Invalid local_uuid_1: {e}"))?;
    let local_uuid_2 = Uuid::from_str(local_uuid_2_str).map_err(|e| format!("Invalid local_uuid_2: {e}"))?;

    Ok(UserUuid {
      local_uuid_1,
      local_uuid_2,
      auth_token: auth_token.to_string(),
    })
  }
}

impl Display for UserUuid {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{local_uuid_1}01g1{auth_token}aabb{local_uuid_2}",
      local_uuid_1 = self.local_uuid_1.simple(),
      auth_token = self.auth_token,
      local_uuid_2 = self.local_uuid_2.simple()
    )
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  #[test]
  fn test_round_trip_examples() {
    let examples = [
      // Original client, original server
      "ba40bc8db65e49b8b6575355281b563f01g169d285fd-bdfc-4b9a-927a-2c94acd8f56eaabb4d0b10ab9d7445f783273c2dd9576885",
      // Stub client, Axel
      "9971109acb5842558c0e26ca94f7042701g1aabb478f207dc9ff462f921f61d16aae73fc",
    ];

    for original in examples {
      let parsed = UserUuid::from_str(original).expect(&format!("failed to parse: {}", original));
      let serialized = parsed.to_string();
      assert_eq!(serialized, original, "round-trip failed for {}", original);
    }
  }
}
