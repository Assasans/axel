use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use flate2::bufread::GzEncoder;
use flate2::Compression;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::io::{BufReader, Read};
use std::sync::{LazyLock, OnceLock};
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::OnceCell;
use tracing::{debug, info, trace, warn};

use crate::call::CallCustom;
use crate::extractor::Params;
use crate::handler::{IntoHandlerResponse, Unsigned};

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
  #[serde(skip)]
  pub master_decompressed: String,
}

impl MasterAllItem {
  pub fn new(master_key: String, master: String) -> Self {
    let digest = md5::compute(&master);
    let reader = BufReader::new(master.as_bytes());
    let mut encoder = GzEncoder::new(reader, Compression::fast());

    let mut compressed = Vec::new();
    encoder.read_to_end(&mut compressed).unwrap();

    Self {
      master_key,
      master: BASE64_STANDARD.encode(compressed),
      checkkey: const_hex::encode(*digest),
      master_decompressed: master,
    }
  }
}

static MASTERS: OnceCell<HashMap<String, MasterAllItem>> = OnceCell::const_new();

async fn load_masters() -> HashMap<String, MasterAllItem> {
  let mut path = env::current_dir().unwrap();
  path.push("master");

  let mut masters_to_load = Vec::new();
  let mut read_dir = tokio::fs::read_dir(&path)
    .await
    .expect(&format!("failed to read masters directory {:?}", path));
  while let Some(master) = read_dir.next_entry().await.unwrap() {
    let is_json = master.path().extension().is_some_and(|extension| extension == "json");
    if !is_json {
      continue;
    }

    masters_to_load.push(master.path());
  }

  info!("loading {} masters concurrently...", masters_to_load.len());
  let start = std::time::Instant::now();

  // Spawn tasks for each master file
  let tasks = masters_to_load
    .into_iter()
    .map(|path| {
      tokio::spawn(async move {
        let name = path
          .with_extension("")
          .file_name()
          .unwrap()
          .to_str()
          .unwrap()
          .to_owned();

        let mut file = File::open(&path)
          .await
          .expect(&format!("failed to open master {:?}", path));
        let mut data = Vec::new();
        file.read_to_end(&mut data).await.unwrap();

        let start = std::time::Instant::now();
        let mut value =
          serde_json::from_slice::<Value>(&mut data).expect(&format!("failed to parse master {:?}", path));
        trace!("parsed master {} in {:?}", name, start.elapsed());
        patch_master(&name, &mut value).await;
        let start = std::time::Instant::now();
        let serialized = serde_json::to_string(&value).unwrap();
        trace!("serialized master {} in {:?}", name, start.elapsed());

        // Execute CPU-intensive operations in a blocking thread
        let master_key = name.clone();
        let master = tokio::task::spawn_blocking(move || MasterAllItem::new(master_key, serialized))
          .await
          .unwrap();

        debug!("loaded master {} (digest: {})", master.master_key, master.checkkey);
        master
      })
    })
    .collect::<Vec<_>>();

  // Wait for all tasks to complete
  let results = join_all(tasks).await;

  // Collect results into HashMap
  let mut masters = HashMap::new();
  for result in results {
    match result {
      Ok(master) => {
        masters.insert(master.master_key.clone(), master);
      }
      Err(e) => {
        warn!("Failed to load master: {:?}", e);
      }
    }
  }

  patch_masters_post(&mut masters).await;

  info!("loaded {} masters in {:?}", masters.len(), start.elapsed());
  masters
}

async fn patch_master(name: &str, value: &mut Value) {
  let now = chrono::Utc::now();

  let start_at = now - chrono::Duration::days(1);
  let start_at_str = start_at.format("%Y/%m/%d %H:%M").to_string();

  let end_at = now + chrono::Duration::days(30);
  let end_at_str = end_at.format("%Y/%m/%d %H:%M").to_string();

  match name {
    // Used for "time remaining" and event ended errors
    "gacha" => {
      info!("patching gacha");
      trace!("end_at_str: {:?}", end_at_str);
      if let Some(array) = value.as_array_mut() {
        for item in array {
          if let Some(item) = item.as_object_mut() {
            if let Some(end_at_value) = item.get_mut("end_at") {
              *end_at_value = Value::String(end_at_str.clone());
            }
          }
        }
      }
    }

    // Used to decide whether to show a gacha in the list
    "gacha_priority" => {
      info!("patching gacha priority");
      if let Some(array) = value.as_array_mut() {
        for item in array {
          if let Some(item) = item.as_object_mut() {
            if let Some(end_at_value) = item.get_mut("end_at") {
              *end_at_value = Value::String(end_at_str.clone());
            }
          }
        }
      }
    }
    "mission_panel_group" => {
      info!("patching mission_panel_group");
      if let Some(array) = value.as_array_mut() {
        for item in array {
          if let Some(item) = item.as_object_mut() {
            if let Some(end_at_value) = item.get_mut("end_at") {
              *end_at_value = Value::String(end_at_str.clone());
            }
            if let Some(reward_end_at_value) = item.get_mut("reward_end_at") {
              *reward_end_at_value = Value::String(end_at_str.clone());
            }
          }
        }
      }
    }
    // jq '.[]' text.json -c -C | grep -i 'new year' | grep -i 'area'
    "event_config" => {
      info!("patching event_config");
      if let Some(array) = value.as_array_mut() {
        for item in array {
          if let Some(item) = item.as_object_mut() {
            // "Etiquette for This New Year's Party!"
            if item.get("event_id").unwrap() != "24011" {
              continue;
            }

            if let Some(start_at_value) = item.get_mut("start_at") {
              *start_at_value = Value::String(start_at_str.clone());
            }
            if let Some(reward_start_at_value) = item.get_mut("reward_start_at") {
              *reward_start_at_value = Value::String(start_at_str.clone());
            }

            if let Some(end_at_value) = item.get_mut("end_at") {
              *end_at_value = Value::String(end_at_str.clone());
            }
            if let Some(reward_end_at_value) = item.get_mut("reward_end_at") {
              *reward_end_at_value = Value::String(end_at_str.clone());
            }
          }
        }
      }
    }
    // Show hidden characters in character list
    // "character" => {
    //   info!("patching character");
    //   if let Some(array) = value.as_array_mut() {
    //     for item in array {
    //       if let Some(item) = item.as_object_mut() {
    //         if let Some(display_start_value) = item.get_mut("display_start") {
    //           *display_start_value = Value::String(start_at_str.clone());
    //         }
    //         if let Some(display_end_value) = item.get_mut("display_end") {
    //           *display_end_value = Value::String(end_at_str.clone());
    //         }
    //       }
    //     }
    //   }
    // }
    "scorechallenge" => {
      info!("patching scorechallenge");
      if let Some(array) = value.as_array_mut() {
        for item in array {
          let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
          if id != "2039" {
            continue;
          }

          if let Some(item) = item.as_object_mut() {
            if let Some(start_at_value) = item.get_mut("start_at") {
              *start_at_value = Value::String(start_at_str.clone());
            }
            if let Some(end_at_value) = item.get_mut("end_at") {
              *end_at_value = Value::String(end_at_str.clone());
            }
          }
        }
      }
    }
    "equip_weapon" => {
      info!("patching equip_weapon");
      if let Some(array) = value.as_array_mut() {
        for item in array {
          if let Some(item) = item.as_object_mut() {
            if let Some(start_at_value) = item.get_mut("start_at") {
              *start_at_value = Value::String(start_at_str.clone());
            }
            if let Some(end_at_value) = item.get_mut("end_at") {
              *end_at_value = Value::String(end_at_str.clone());
            }
            if let Some(end_at_value) = item.get_mut("display_end") {
              *end_at_value = Value::String(end_at_str.clone());
            }
          }
        }
      }
    }
    _ => {
      // let mut logged = false;
      // if let Some(array) = value.as_array_mut() {
      //   for item in array {
      //     // if let Some(item) = item.as_object_mut() {
      //     //   for (key, value) in item.iter_mut() {
      //     //     if key.contains("start") {
      //     //       continue;
      //     //     }
      //     //     if let Value::String(s) = value {
      //     //       if s.contains("2024/5/6") {
      //     //         *value = Value::String(end_at_str);
      //     //         info!("patched master {}: {}", name, key);
      //     //       }
      //     //     }
      //     //   }
      //     // }
      //
      //     if let Some(start_at_value) = item.get_mut("start_at") {
      //       if !logged {
      //         info!("patching 'start_at' generic master {}", name);
      //         logged = true;
      //       }
      //
      //       *start_at_value = Value::String(start_at_str.clone());
      //     }
      //     if let Some(start_at_value) = item.get_mut("start") {
      //       if !logged {
      //         info!("patching 'start' generic master {}", name);
      //         logged = true;
      //       }
      //     }
      //     if let Some(start_at_value) = item.get_mut("display_start") {
      //       if !logged {
      //         info!("patching 'display_start' generic master {}", name);
      //         logged = true;
      //       }
      //
      //       *start_at_value = Value::String(start_at_str.clone());
      //     }
      //
      //     if let Some(end_at_value) = item.get_mut("end_at") {
      //       if !logged {
      //         info!("patching 'end_at' generic master {}", name);
      //         logged = true;
      //       }
      //
      //       *end_at_value = Value::String(end_at_str.clone());
      //     }
      //     if let Some(end_at_value) = item.get_mut("end") {
      //       if !logged {
      //         info!("patching 'end' generic master {}", name);
      //         logged = true;
      //       }
      //
      //       *end_at_value = Value::String(end_at_str.clone());
      //     }
      //     if let Some(end_at_value) = item.get_mut("display_end") {
      //       if !logged {
      //         info!("patching 'display_end' generic master {}", name);
      //         logged = true;
      //       }
      //
      //       *end_at_value = Value::String(end_at_str.clone());
      //     }
      //   }
      // }
    }
  }
}

async fn patch_masters_post(masters: &mut HashMap<String, MasterAllItem>) {
  let gacha_master = masters.get("gacha").expect("gacha master not found");
  let gacha_items = serde_json::from_str::<Vec<GachaMasterItem>>(&gacha_master.master_decompressed).unwrap();
  let mut gacha_priority_items = Vec::new();
  for item in gacha_items {
    gacha_priority_items.push(GachaPriorityMasterItem {
      id: item.gacha_id.clone(),
      priority: "0".to_string(),
      start_at: item.start_at.clone(),
      end_at: item.end_at.clone(),
      gacha_id: item.gacha_id,
    });
  }
  masters.insert(
    "gacha_priority".to_string(),
    MasterAllItem::new(
      "gacha_priority".to_string(),
      serde_json::to_string(&gacha_priority_items).unwrap(),
    ),
  );
  info!("generated synthetic gacha_priority");
}

#[derive(Debug, Serialize, Deserialize)]
struct GachaPriorityMasterItem {
  pub id: String,
  pub priority: String,
  pub start_at: String,
  pub end_at: String,
  pub gacha_id: String,
}

#[derive(Serialize, Deserialize)]
struct GachaMasterItem {
  pub gacha_id: String,
  pub name: String,
  pub description_text: String,
  pub enable: String,
  pub start_at: String,
  pub end_at: String,
  pub limit: String,
  pub limit_x: String,
  pub banner_url: String,
  pub movie: String,
  pub introduction_image: String,
  pub info_banner_url: String,
  pub gacha_info_url: String,
  pub other_tab_priority: String,
  pub footer_banner: String,
  pub footer_banner_order: String,
  pub limit_id: String,
  pub exchang_id: String,
  pub story: String,
  pub story_fes: String,
  pub ratio_text_pattern: String,
  pub exchang_bonus_id: String,
  pub exchang_name: String,
  pub always_display_flag: String,
  pub step_loop_count: String,
  pub result_bg: String,
  pub card_img_2: String,
  pub card_img_3: String,
  pub card_img_4: String,
  pub home_appeal_priority: String,
}

#[deprecated(note = "Use get_master_manager() instead, parsing each time manually is slow")]
pub async fn get_masters() -> &'static HashMap<String, MasterAllItem> {
  MASTERS.get_or_init(load_masters).await
}

#[deprecated(note = "Use get_master_manager() instead, parsing each time manually is slow")]
pub fn get_masters_definitely_initialized() -> &'static HashMap<String, MasterAllItem> {
  MASTERS.get().expect("masters not loaded yet")
}

#[derive(Debug, Deserialize)]
pub struct MasterAllRequest {
  #[serde(deserialize_with = "crate::serde_compat::comma_separated_string")]
  pub master_keys: Vec<String>,
}

pub async fn master_all(Params(params): Params<MasterAllRequest>) -> impl IntoHandlerResponse {
  debug!(params = ?params.master_keys, "loading masters");
  let masters = get_masters().await;
  let masters = params
    .master_keys
    .iter()
    .map(|key| masters.get(key).expect(&format!("master {:?} not found", key)))
    .cloned()
    .collect::<Vec<_>>();
  Unsigned(MasterAll {
    masterversion: "202408050001".to_owned(),
    masterarray: masters,
    compressed: true,
  })
}

pub struct MasterManager {
  masters: HashMap<String, Vec<Value>>,
}

impl MasterManager {
  pub fn new(opaque_masters: &HashMap<String, MasterAllItem>) -> Self {
    // Large and not needed on server side
    let blacklist = ["pack", "assetname", "text", "voice", "navi"];

    let start = Instant::now();
    let mut masters = HashMap::new();
    for (key, item) in opaque_masters.iter() {
      if blacklist.contains(&key.as_str()) {
        continue;
      }

      let value = serde_json::from_str::<Vec<Value>>(&item.master_decompressed).unwrap();
      masters.insert(key.clone(), value);
    }
    info!("MasterManager initialized in {:?}", start.elapsed());

    Self { masters }
  }

  pub fn get_master(&self, key: &str) -> &Vec<Value> {
    self.masters.get(key).expect(&format!("master {:?} not found", key))
  }
}

pub static MASTER_MANAGER: OnceLock<MasterManager> = OnceLock::new();

pub fn get_master_manager() -> &'static MasterManager {
  MASTER_MANAGER.get().expect("master manager not initialized yet")
}
