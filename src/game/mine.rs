use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use dashmap::DashMap;
use foundations::BootstrapResult;
use futures::Stream;
use fxhash::FxHashSet;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use rand::{rngs::OsRng, SeedableRng};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use uuid::Uuid;

use crate::{
  db::{get_available_inventory_space, NewInventoryItem},
  protos::StartMiningResponse,
};

use super::items::mine_locations;

#[derive(Clone)]
struct MiningSession {
  token: Uuid,
  stop_tx: Arc<mpsc::Sender<StopMiningReason>>,
  location_name: &'static str,
}

lazy_static! {
  static ref ACTIVE_MINING_SESSIONS: DashMap<i32, MiningSession> = DashMap::new();
}

static INVENTORY_ITEM_SAVE_TX: OnceCell<mpsc::Sender<NewInventoryItem>> = OnceCell::new();

pub enum StopMiningReason {
  Manual,
  InventoryFull,
}

fn inventory_item_save_tx() -> &'static mpsc::Sender<NewInventoryItem> {
  INVENTORY_ITEM_SAVE_TX
    .get()
    .expect("Inventory item saver not initialized")
}

fn check_inventory_space(user_ids: Vec<i32>) -> impl std::future::Future<Output = ()> {
  async move {
    let mut unique_user_ids = FxHashSet::default();
    unique_user_ids.extend(user_ids);

    for user_id in unique_user_ids {
      let available_inventory_space = get_available_inventory_space(user_id).await.unwrap_or(0);
      if available_inventory_space <= 0 {
        warn!("User {user_id} inventory full; stopping mining session");
        stop_mining(user_id, StopMiningReason::InventoryFull, None);
      }
    }
  }
}

pub async fn start_inventory_item_saver() -> BootstrapResult<()> {
  let (tx, mut rx) = mpsc::channel(10);
  INVENTORY_ITEM_SAVE_TX
    .set(tx)
    .map_err(|_| anyhow::anyhow!("Inventory item saver already started"))?;

  tokio::task::spawn(async move {
    let mut last_save_time = Instant::now();
    let mut items_to_save = Vec::new();
    let mut unique_user_ids = Vec::default();

    loop {
      let res = tokio::time::timeout(tokio::time::Duration::from_millis(1350), rx.recv()).await;
      if let Ok(Some(item)) = res {
        items_to_save.push(item);
      }

      if items_to_save.is_empty() {
        continue;
      }

      if last_save_time.elapsed().as_secs() >= 2 || items_to_save.len() >= 100 {
        unique_user_ids.extend(items_to_save.iter().map(|item| item.user_id));
        unique_user_ids.sort_unstable();
        unique_user_ids.dedup();

        match crate::db::insert_inventory_items(&items_to_save).await {
          Err(err) => {
            error!("Failed to save inventory items: {err:?}");
          },
          Ok(_) => {
            items_to_save.clear();
            last_save_time = Instant::now();
          },
        }

        tokio::task::spawn(check_inventory_space(std::mem::take(&mut unique_user_ids)));
      }
    }
  });

  info!("Inventory item saver started");

  Ok(())
}

struct MineSessionDropHandle {
  user_id: i32,
  session_token: Uuid,
}

impl Drop for MineSessionDropHandle {
  fn drop(&mut self) {
    stop_mining(
      self.user_id,
      StopMiningReason::Manual,
      Some(self.session_token),
    );
  }
}

pub async fn start_mining(
  user_id: i32,
  location_name: &str,
  session_token: Option<Uuid>,
) -> Result<impl Stream<Item = Result<StartMiningResponse, Status>>, Status> {
  let session_token = session_token.unwrap_or_else(Uuid::new_v4);
  let drop_handle = MineSessionDropHandle {
    user_id,
    session_token,
  };

  let location = match mine_locations()
    .iter()
    .find(|loc| loc.descriptor.name == location_name)
  {
    Some(loc) => loc,
    None => return Err(Status::invalid_argument("Invalid mine location")),
  };
  let loot_table = &location.loot_table;
  let location_name: &'static str = &location.descriptor.name;

  let available_inventory_space = get_available_inventory_space(user_id)
    .await
    .map_err(|err| {
      error!("Failed to get available inventory space: {err}");
      Status::internal("Internal DB error")
    })?;
  if available_inventory_space <= 0 {
    return Err(Status::resource_exhausted(
      "Inventory is full; mining halted.  Upgrade storage capacity or remove items from inventory \
       before mining more.",
    ));
  }

  let (stop_tx, mut stop_rx) = mpsc::channel(1);
  let session = MiningSession {
    token: session_token,
    stop_tx: Arc::new(stop_tx),
    location_name,
  };
  ACTIVE_MINING_SESSIONS.insert(user_id, session.clone());

  crate::metrics::game::active_mine_sessions(location_name).inc();

  let (tx, rx) = mpsc::channel(10);
  let mut rng = pcg_rand::Pcg64::from_rng(OsRng).unwrap();

  info!("User {user_id} started mining at location {location_name}");

  tokio::task::spawn(async move {
    let millis_until_next_loot = 8200u32;

    if tx
      .send(Ok(StartMiningResponse {
        loot: None,
        millis_until_next_loot,
      }))
      .await
      .is_err()
    {
      return;
    }

    loop {
      tokio::time::sleep(Duration::from_millis(millis_until_next_loot as _)).await;

      if let Ok(stop_reason) = stop_rx.try_recv() {
        match stop_reason {
          StopMiningReason::Manual => info!("User {user_id} stopped mining manually"),
          StopMiningReason::InventoryFull => {
            warn!("User {user_id} stopped mining due to full inventory");
            let _ = tx
              .send(Err(Status::resource_exhausted(
                "Inventory is full; mining halted.  Upgrade storage capacity or remove items from \
                 inventory before continuing.",
              )))
              .await;
            break;
          },
        }
        break;
      }

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
        .send(Ok(StartMiningResponse {
          loot: Some(loot),
          millis_until_next_loot,
        }))
        .await
        .is_err()
      {
        break;
      }
    }

    drop(drop_handle);
  });

  Ok(ReceiverStream::new(rx))
}

pub fn stop_mining(user_id: i32, reason: StopMiningReason, session_token: Option<Uuid>) {
  let removed = ACTIVE_MINING_SESSIONS.remove_if(&user_id, |_, session| match session_token {
    Some(token) => session.token == token,
    None => true,
  });

  if let Some((_uid, session)) = removed {
    let _ = session.stop_tx.send(reason);
    crate::metrics::game::active_mine_sessions(session.location_name).dec();
  }
}
