use tonic::Status;

use crate::{
  db::{debit_user_inventory, get_user_upgrades, pool},
  protos::{ItemCost, UpgradeBaseRequest, UpgradeType, Upgrades},
};

use super::items::get_item_id_by_name;

pub const BASE_INVENTORY_SIZE: u32 = 5_000;
pub const INVENTORY_CAPACITY_PER_UPGRADE: u32 = 1_000;

pub fn get_inventory_upgrade_cost(level: u32) -> [ItemCost; 3] {
  let base_cost = (3. * (level + 1) as f32) * 1.18_f32.powf(0.33 * level as f32);

  [
    ItemCost {
      item_id: get_item_id_by_name("wooden_palette"),
      total_quality: base_cost * 1.25,
    },
    ItemCost {
      item_id: get_item_id_by_name("wooden_beam"),
      total_quality: base_cost,
    },
    ItemCost {
      item_id: get_item_id_by_name("roof_shingles"),
      total_quality: base_cost * 0.8,
    },
  ]
}

pub async fn upgrade_inventory_storage(user_id: i32) -> Result<(), Status> {
  let mut txn = pool().begin().await.map_err(|err| {
    error!("Failed to start transaction: {err}");
    Status::internal("Internal DB error")
  })?;

  // Insert a row if it doesn't exist
  sqlx::query!(
    "INSERT INTO bases (user_id) VALUES ($1) ON CONFLICT DO NOTHING",
    user_id,
  )
  .execute(&mut *txn)
  .await
  .map_err(|err| {
    error!("Failed to insert base row: {err}");
    Status::internal("Internal DB error")
  })?;

  let inventory_upgrade_level = sqlx::query_scalar!(
    "SELECT storage_level FROM bases WHERE user_id = $1 FOR UPDATE",
    user_id,
  )
  .fetch_optional(&mut *txn)
  .await
  .map(|row| row.unwrap_or(0))
  .map_err(|err| {
    error!("Failed to fetch inventory upgrade level: {err}");
    Status::internal("Internal DB error")
  })? as u32;

  let upgrade_cost = get_inventory_upgrade_cost(inventory_upgrade_level);

  debit_user_inventory(&mut txn, user_id, &upgrade_cost).await?;

  sqlx::query!(
    "UPDATE bases SET storage_level = storage_level + 1 WHERE user_id = $1",
    user_id,
  )
  .execute(&mut *txn)
  .await
  .map_err(|err| {
    error!("Failed to upgrade inventory storage: {err}");
    Status::internal("Internal DB error")
  })?;

  txn.commit().await.map_err(|err| {
    error!("Failed to commit transaction: {err}");
    Status::internal("Internal DB error")
  })?;

  info!(
    "Successfully upgraded inventory storage for user {user_id} to level {}",
    inventory_upgrade_level + 1
  );

  Ok(())
}

pub(crate) async fn upgrade_base(
  user_id: i32,
  req: UpgradeBaseRequest,
) -> Result<Upgrades, Status> {
  match UpgradeType::try_from(req.upgrade_type) {
    Ok(UpgradeType::Storage) => upgrade_inventory_storage(user_id).await,
    Err(_) => {
      error!("Invalid upgrade type: {}", req.upgrade_type);
      return Err(Status::invalid_argument("Invalid upgrade type"));
    },
  }?;

  get_user_upgrades(user_id).await.map_err(|err| {
    error!("Failed to fetch user upgrades: {err}");
    Status::internal("Internal DB error")
  })
}
