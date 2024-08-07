use std::collections::HashMap;
use std::env;
use std::io::{BufReader, Read};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use flate2::bufread::GzEncoder;
use flate2::Compression;
use jwt_simple::prelude::Serialize;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::OnceCell;
use tracing::{debug, info};

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
  pub fn new(master_key: String, master: String) -> Self {
    let digest = md5::compute(&master);
    let reader = BufReader::new(master.as_bytes());
    let mut encoder = GzEncoder::new(reader, Compression::default());

    let mut compressed = Vec::new();
    encoder.read_to_end(&mut compressed).unwrap();

    Self {
      master_key,
      master: BASE64_STANDARD.encode(compressed),
      checkkey: hex::encode(&*digest),
    }
  }
}

static MASTERS: OnceCell<HashMap<String, MasterAllItem>> = OnceCell::const_new();

async fn load_masters() -> HashMap<String, MasterAllItem> {
  let mut path = env::current_dir().unwrap();
  path.push("master");

  info!("loading masters...");
  let mut masters = HashMap::new();
  let mut read_dir = tokio::fs::read_dir(&path)
    .await
    .expect(&format!("failed to read masters directory {:?}", path));
  while let Some(master) = read_dir.next_entry().await.unwrap() {
    let is_json = master.path().extension().is_some_and(|extension| extension == "json");
    if !is_json {
      continue;
    }

    let mut file = File::open(master.path())
      .await
      .expect(&format!("failed to open master {:?}", master));
    let mut data = String::new();
    file.read_to_string(&mut data).await.unwrap();
    // let data = serde_json::from_str(&data).expect(&format!("failed to parse master {:?}", master));
    let master = MasterAllItem::new(
      master
        .path()
        .with_extension("")
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned(),
      data,
    );
    debug!("loaded master {} (digest: {})", master.master_key, master.checkkey);
    masters.insert(master.master_key.to_owned(), master);
  }

  info!("loaded {} masters", masters.len());
  masters
}

pub async fn get_masters() -> &'static HashMap<String, MasterAllItem> {
  MASTERS.get_or_init(load_masters).await
}
