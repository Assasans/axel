use std::sync::LazyLock;

use jwt_simple::prelude::Serialize;

use crate::call::CallCustom;

#[derive(Debug, Serialize)]
pub struct MasterAll {
  pub masterversion: String,
  pub masterarray: Vec<MasterAllItem>,
  #[serde(with = "crate::bool_as_int")]
  pub compressed: bool,
}

impl CallCustom for MasterAll {}

#[derive(Clone, Debug, Serialize)]
pub struct MasterAllItem {
  pub master_key: String,
  pub master: String,
  pub checkkey: String,
}

impl MasterAllItem {
  pub fn new(master_key: String, master: String, checkkey: String) -> Self {
    Self {
      master_key,
      master,
      checkkey,
    }
  }
}

// This is a gzip compressed JSON, checkkey is MD5 of uncompressed data
pub static MASTERS: LazyLock<Vec<MasterAllItem>> = LazyLock::new(|| {
  vec![
    MasterAllItem::new(
      "system".to_owned(),
      r#"H4sIAAAAAAAAA71X2Y7qRhD9l3nI0w2D8cLMlUb3Q0LUauwC+tKL0wszJMq/p7yAq70gIkV5s+U+tZ4+Vf7trxdRvXx/SV6+vZzhik8enMeXC5cB+tfu+e9v/dnN/awNEi5gyfF0OJYOJvkXs9wDOZesh4PZ/WB54vYITAolPJOeJQWB5Os1AeWLoM068hOhijGKS7Ce+ZMFdzKyGiMJdHuHVnDgAR2djAKmQO2jCiTrJEkoMInrxYKVrOb+FBXZKrd7FbqCr9XJK0nQQxmdN7aDc11ZI2i0J+9r9333unutJb+ujsYcJaxKo3avLWz3yusaXVTguZDuh6g+8ONKw5fRq7PRxoU9J16zkVcLFwGf/5PzfCZlYdysR+GDBrdCBzeXwbX+8FOV5Jtk+57mRfZD+Y834qJYzu+/8PQLL70w+uPTCg+/dsaJ94FLijtsfsU9Z954LpkTf8KEhRGd3u5gaUouHXgv9PFfmXgfGAnKeCgRLM1xgszHyPZu9RfIioNAOnvLtcOXukmYKbzqIyubqZVBbVA+tGdND67sGEQFrDRBUwHKCW4zxoX9z3Dkred5dEHQ6Qgt9IW7ZSh1TK4D6IrtufeYOregOWsKNysgm3HaA6/3kpdnh4J1woohAQS2TQK/AEvizhF08QR6Q+seoQnlULvELQVrjMLycx8cCzWSCLAsaBKNzKv6ZqDfwYq2GAAHdhSXqOfU9fsswvPz0kBI17MI5NZScdKBUQdhnWc9Du8zgdBP82qbDgyrS8arC9clMoOrmuFFViySvYTg0sc4f61hvp5p9hipg6J1jbPOF7E18rHlOA04pxETBTyLmnlRnsG397cGi3r4R4gXgdjzQKd2CgotfMPCbhpib+WZtYo0vlQbYuMttjGPbaORxtQLhX+PjWhjFcbxhBESSTbwrQr6CCgJe9BwwH0CR4M0Hi/EtVsw5puYDbRxJY4TXC6kBLTE4KstqeU4CK5sbzTes84AiWRFpCZLJ7FYOAbJLRusjkPZEny2iO8LjIyKVIaMxSx/BkxFJiG9yIpn0Gk0YAh6+0TiN56NS0h78Tax0xIcx9R5nkPZwCEFzQCtuOLobH+90Qm1EhV3Hp2vF9EO53IzIZtN0zdRyAVRzZNFG51aL5igYWwWTXDnhHsMTufBthmtbSY1lE3dxzbIJcqz+A6QrnXvY9Lmk70kp4sfVwKHa3C3KMZwOjDygXmNcAkLVe+06To7GMsOXPVvgaoAIWA+EBD3ilvVYv2lNRt41i8jT2WdTbMm87EJshXeW/jwyW3FjtaEGsk0H3gxMBDPSdwHAVcajsl6wEKYsgwWNxUcDRW/unkGFglJ3tZWYN2xAYLdF7TyPM2F4BcVUAnn2i0LW2pHAymhTSzSufonLGjZOL83cJ5+RT5bxW6zOUXTI/ohLB7AJteerjXF9gESb48oH8PfHsDxDxO0W1qoinnCEM+P8duBL/0W2G8ACkyIhjUh2TaJnXajTOgSV2ek1pIrsk/xTj26RtbGOeh4MZmrFJ/GXvFHw6CJpTVsm42Oh+ZnTU0WIQoZ8aYv4y29SlxEFZr9ehRlVBuyXLfyyauql9DY6e//AAMkNU5vEQAA"#.to_owned(),
      "e1e97780473ebe7a4c83b822bc1087c3".to_owned(),
    ),
    MasterAllItem::new("pack_mission_progress".to_owned(), r#"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"#.to_owned(), "6688913fbcb515dc75084add264c7976".to_owned()),
  ]
});
