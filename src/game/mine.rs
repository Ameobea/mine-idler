use std::time::Instant;

use dashmap::DashMap;
use foundations::BootstrapResult;
use futures::Stream;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use rand::{rngs::OsRng, SeedableRng};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use uuid::Uuid;

use crate::{db::NewInventoryItem, protos::StartMiningResponse};

use super::items::mine_locations;

#[derive(Clone)]
struct MiningSession {
  token: Uuid,
}

lazy_static! {
  static ref ACTIVE_MINING_SESSIONS: DashMap<i32, MiningSession> = DashMap::new();
}

static INVENTORY_ITEM_SAVE_TX: OnceCell<mpsc::Sender<NewInventoryItem>> = OnceCell::new();

fn inventory_item_save_tx() -> &'static mpsc::Sender<NewInventoryItem> {
  INVENTORY_ITEM_SAVE_TX
    .get()
    .expect("Inventory item saver not initialized")
}

pub async fn start_inventory_item_saver() -> BootstrapResult<()> {
  let (tx, mut rx) = mpsc::channel(10);
  INVENTORY_ITEM_SAVE_TX
    .set(tx)
    .map_err(|_| anyhow::anyhow!("Inventory item saver already started"))?;

  tokio::task::spawn(async move {
    let mut last_save_time = Instant::now();
    let mut items_to_save = Vec::new();

    loop {
      let res = tokio::time::timeout(tokio::time::Duration::from_millis(1350), rx.recv()).await;
      if let Ok(Some(item)) = res {
        items_to_save.push(item);
      }

      if items_to_save.is_empty() {
        continue;
      }

      if last_save_time.elapsed().as_secs() >= 2 {
        match crate::db::insert_inventory_items(&items_to_save).await {
          Err(err) => {
            error!("Failed to save inventory items: {err:?}");
          },
          Ok(_) => {
            items_to_save.clear();
            last_save_time = Instant::now();
          },
        }
      }
    }
  });

  info!("Inventory item saver started");

  Ok(())
}

pub async fn start_mining(
  user_id: i32,
  location_name: &str,
) -> Result<impl Stream<Item = StartMiningResponse>, Status> {
  let session = MiningSession {
    token: Uuid::new_v4(),
  };
  ACTIVE_MINING_SESSIONS.insert(user_id, session.clone());

  let loot_table = match mine_locations()
    .iter()
    .find(|loc| loc.descriptor.name == location_name)
  {
    Some(loc) => &loc.loot_table,
    None => return Err(Status::invalid_argument("Invalid mine location")),
  };

  let (tx, rx) = mpsc::channel(10);
  let mut rng = pcg_rand::Pcg64::from_rng(OsRng).unwrap();

  info!("User {user_id} started mining at location {location_name}");

  tokio::task::spawn(async move {
    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;

      // Check if this session is still active
      match ACTIVE_MINING_SESSIONS.get(&user_id) {
        Some(o_session) if session.token == o_session.token => {},
        _ => {
          info!("A different mining session has started for user {user_id}; stopping old session");
          break;
        },
      }

      let loot = loot_table.roll(&mut rng);

      let res = inventory_item_save_tx()
        .send(NewInventoryItem {
          user_id,
          item_id: loot.id,
          quality: loot.quality,
          value: loot.value,
          modifiers: None, // TODO
        })
        .await;
      if res.is_err() {
        error!("Failed to save inventory item; channel closed");
        break;
      }

      if tx
        .send(StartMiningResponse { loot: Some(loot) })
        .await
        .is_err()
      {
        break;
      }
    }
  });

  Ok(ReceiverStream::new(rx))
}

pub async fn stop_mining(user_id: i32) { ACTIVE_MINING_SESSIONS.remove(&user_id); }
