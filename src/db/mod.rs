use std::{cmp::Reverse, time::Duration};

use foundations::BootstrapResult;
use fxhash::FxHashMap;
use once_cell::sync::OnceCell;
use sqlx::{
  pool::PoolOptions,
  postgres::{PgConnectOptions, PgQueryResult},
  FromRow, Pool, Postgres,
};
use tonic::Status;
use uuid::Uuid;

use crate::{
  auth::hash_password,
  conf::Settings,
  game::{
    items::{get_item_display_name_by_id, populate_items_table},
    upgrades::{get_inventory_upgrade_cost, BASE_INVENTORY_SIZE, INVENTORY_CAPACITY_PER_UPGRADE},
  },
  protos::{
    AggregatedInventory, AggregatedItemCount, HiscoreEntry, Item, ItemCost, ItemDescriptor,
    ItemQualityHistogram, SortBy, SortDirection, StorageUpgrades, Upgrades, UserAccountInfo,
  },
};

static DB_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();

pub async fn init_db(settings: &Settings) -> BootstrapResult<()> {
  let pool_options = PoolOptions::new()
    .max_connections(20)
    .min_connections(1)
    .max_lifetime(Duration::from_secs(600));
  let pool = pool_options
    .connect_with(
      PgConnectOptions::new()
        .host(&settings.database.host)
        .port(settings.database.port)
        .username(&settings.database.username)
        .password(&settings.database.password)
        .database(&settings.database.database),
    )
    .await?;
  info!("Database pool initialized");

  DB_POOL
    .set(pool)
    .map_err(|_| anyhow::anyhow!("Database pool already initialized"))?;

  populate_items_table().await?;

  Ok(())
}

pub fn pool() -> &'static Pool<Postgres> { DB_POOL.get().expect("Database pool not initialized") }

/// If the session token is valid, returns the ID of the logged-in user.
pub async fn validate_session_token(
  session_token: &str,
  session_token_lifetime: Duration,
) -> Result<Option<i32>, Status> {
  let session = sqlx::query!(
    "SELECT user_id, created_at FROM sessions WHERE token = $1",
    session_token,
  )
  .fetch_optional(pool())
  .await
  .map_err(|err| {
    error!("Error reading session from database: {err}");
    Status::internal("Internal DB error")
  })?;

  let Some(session) = session else {
    return Ok(None);
  };

  let created_at = session.created_at;
  let now = chrono::Utc::now().naive_utc();
  let expiry = created_at + chrono::Duration::from_std(session_token_lifetime).unwrap();
  if now > expiry {
    return Err(Status::unauthenticated("Session token expired"));
  }

  Ok(Some(session.user_id))
}

pub async fn get_hashed_password(username: &str) -> Result<Option<(i32, String)>, Status> {
  let record = sqlx::query!(
    "SELECT id, hashed_password FROM users WHERE username = $1",
    username
  )
  .fetch_optional(pool())
  .await
  .map_err(|err| {
    error!("Error reading user from database: {err}");
    Status::internal("Internal DB error")
  })?;

  Ok(record.map(|row| (row.id, row.hashed_password)))
}

pub async fn insert_session_token(user_id: i32, session_token: &str) -> Result<(), Status> {
  sqlx::query!(
    "INSERT INTO sessions (user_id, token) VALUES ($1, $2)",
    user_id,
    session_token
  )
  .execute(pool())
  .await
  .map_err(|err| {
    error!("Error inserting session token: {err}");
    Status::internal("Internal DB error")
  })?;

  Ok(())
}

/// Adds a new user to the database with the provided username and password, returning the ID of the
/// new user if successful.
pub async fn insert_new_user(username: &str, password: &str) -> Result<i32, Status> {
  let hashed_password = hash_password(password).map_err(|err| {
    error!("Error hashing password: {err}");
    Status::internal("Internal error")
  })?;

  let user_id = sqlx::query!(
    "INSERT INTO users (username, hashed_password) VALUES ($1, $2) RETURNING id",
    username,
    hashed_password
  )
  .fetch_one(pool())
  .await
  .map(|row| row.id)
  .map_err(|err| {
    // Handle unique constraint violation
    if let Some(err) = err.as_database_error() {
      if err.constraint().is_some() {
        warn!("Tried to register user that already exists: {username}");
        return Status::already_exists("User already exists");
      }
    }

    error!("Error inserting new user: {err}");
    Status::internal("Internal error registering user")
  })?;

  sqlx::query!(
    "INSERT INTO bases (user_id) VALUES ($1) ON CONFLICT DO NOTHING",
    user_id,
  )
  .execute(pool())
  .await
  .map_err(|err| {
    error!("Failed to insert base row: {err}");
    Status::internal("Internal DB error")
  })?;

  Ok(user_id)
}

pub async fn insert_item_descriptors(items: &[ItemDescriptor]) -> Result<(), Status> {
  for item in items {
    sqlx::query!(
      "INSERT INTO items (id, name, description, rarity_tier) VALUES ($1, $2, $3, $4) ON CONFLICT \
       (id) DO UPDATE SET name = $2, description = $3, rarity_tier = $4;",
      item.id as i64,
      item.name,
      item.description,
      item.rarity_tier as i64
    )
    .execute(pool())
    .await
    .map_err(|err| {
      error!("Error inserting item {}: {err}", item.id);
      Status::internal("Internal DB error inserting items")
    })?;
  }

  Ok(())
}

pub struct NewInventoryItem {
  pub user_id: i32,
  pub item_id: i32,
  pub quality: f32,
  pub value: f32,
  pub modifiers: Option<serde_json::Value>,
}

pub async fn insert_inventory_items(items: &[NewInventoryItem]) -> sqlx::Result<PgQueryResult> {
  let user_ids: Vec<i32> = items.iter().map(|item| item.user_id).collect();
  let item_ids: Vec<i32> = items.iter().map(|item| item.item_id).collect();
  let qualities: Vec<f32> = items.iter().map(|item| item.quality).collect();
  let values: Vec<f32> = items.iter().map(|item| item.value).collect();
  let modifiers: Vec<Option<serde_json::Value>> =
    items.iter().map(|item| item.modifiers.clone()).collect();

  sqlx::query!(
    "INSERT INTO inventory (user_id, item_id, quality, value, modifiers) SELECT * FROM \
     UNNEST($1::int4[], $2::int4[], $3::float4[], $4::float4[], $5::jsonb[])",
    &user_ids,
    &item_ids,
    &qualities,
    &values,
    &modifiers as &[Option<serde_json::Value>]
  )
  .execute(pool())
  .await
}

pub async fn get_user_account(user_id: i32) -> sqlx::Result<Option<UserAccountInfo>> {
  sqlx::query_as!(
    UserAccountInfo,
    "SELECT id, username FROM users WHERE id = $1",
    user_id
  )
  .fetch_optional(pool())
  .await
}

#[derive(FromRow)]
pub struct DbItem {
  id: Uuid,
  item_id: i32,
  quality: f32,
  value: f32,
  modifiers: Option<serde_json::Value>,
}

pub(crate) async fn get_user_inventory(
  user_id: i32,
  page_size: u32,
  page_number: u32,
  sort_by: SortBy,
  sort_direction: SortDirection,
) -> Result<Vec<Item>, Status> {
  let sort_column = match sort_by {
    SortBy::DateAcquired => "inv.created_at",
    SortBy::RarityTier => "i.rarity_tier",
    SortBy::Value => "inv.value",
  };
  let sort_direction = match sort_direction {
    SortDirection::Ascending => "ASC",
    SortDirection::Descending => "DESC",
  };

  let items: Vec<DbItem> = sqlx::query_as(&format!(
    "SELECT inv.id, inv.item_id, inv.quality, inv.value, inv.modifiers FROM inventory inv JOIN \
     items i ON inv.item_id = i.id WHERE inv.user_id = $1 ORDER BY {sort_column} {sort_direction} \
     LIMIT $2 OFFSET $3"
  ))
  .bind(user_id)
  .bind(page_size.clamp(0, 1000) as i32)
  .bind((page_number * page_size) as i32)
  .fetch_all(pool())
  .await
  .map_err(|err| {
    error!("Error reading user inventory from database: {err}");
    Status::internal("Internal DB error fetching inventory")
  })?;

  Ok(
    items
      .into_iter()
      .map(|item| -> Result<_, Status> {
        let modifiers = item
          .modifiers
          .into_iter()
          .map(|modifier| serde_json::from_value(modifier))
          .collect::<Result<_, _>>()
          .map_err(|err| {
            error!("Found item with un-parseable modifiers in DB: {err}");
            Status::internal("Internal DB error fetching inventory")
          })?;

        Ok(Item {
          id: item.item_id,
          quality: item.quality,
          value: item.value,
          modifiers,
        })
      })
      .collect::<Result<_, _>>()?,
  )
}

pub async fn get_user_aggregated_inventory(user_id: i32) -> sqlx::Result<AggregatedInventory> {
  // We want total item count for each item ID in the user's inventory along with histogram buckets
  // for the quality distribution of each item.
  //
  // Quality is hard-capped from [0,1], and we used 32 fixed buckets for now.
  let rows = sqlx::query!(
    "SELECT item_id, COUNT(*) as total_count, SUM(quality) as total_quality, SUM(value) as \
     total_value, width_bucket(quality, 0, 1, 32) AS quality_bucket_ix FROM inventory WHERE \
     user_id = $1 GROUP BY item_id, quality_bucket_ix",
    user_id
  )
  .fetch_all(pool())
  .await?;

  struct QualityBucket {
    bucket_ix: u32,
    total_count: u32,
    total_quality: f32,
    total_value: f32,
  }

  let mut counts_by_item_id: FxHashMap<i32, Vec<QualityBucket>> = FxHashMap::default();
  for row in rows {
    let item_id = row.item_id as i32;
    let quality_bucket_ix = row.quality_bucket_ix.unwrap_or(0) as u32;
    let total_count = row.total_count.unwrap_or(0);
    let total_quality = row.total_quality.unwrap_or(0.);
    let total_value = row.total_value.unwrap_or(0.);

    counts_by_item_id
      .entry(item_id)
      .or_insert_with(Vec::new)
      .push(QualityBucket {
        bucket_ix: quality_bucket_ix,
        total_count: total_count as _,
        total_quality,
        total_value,
      });
  }

  for bucket in counts_by_item_id.values_mut() {
    for i in 1..=32 {
      if !bucket.iter().any(|b| b.bucket_ix == i) {
        bucket.push(QualityBucket {
          bucket_ix: i,
          total_count: 0,
          total_quality: 0.,
          total_value: 0.,
        });
      }
    }
    bucket.sort_unstable_by_key(|b| b.bucket_ix);
  }

  let mut item_counts: Vec<_> = counts_by_item_id
    .into_iter()
    .map(|(item_id, buckets)| AggregatedItemCount {
      item_id: item_id as _,
      total_count: buckets.iter().map(|b| b.total_count).sum(),
      total_quality: buckets.iter().map(|b| b.total_quality).sum(),
      total_value: buckets.iter().map(|b| b.total_value).sum(),
      quality_histogram: Some(ItemQualityHistogram {
        buckets: buckets
          .into_iter()
          .map(|bucket| bucket.total_count)
          .collect(),
      }),
    })
    .collect();
  item_counts.sort_unstable_by_key(|count| Reverse(count.total_count));

  Ok(AggregatedInventory { item_counts })
}

pub async fn get_hiscores() -> sqlx::Result<Vec<HiscoreEntry>> {
  let rows = sqlx::query!(
    "SELECT u.username, SUM(inv.value) AS total_value FROM inventory inv INNER JOIN users u ON \
     inv.user_id = u.id GROUP BY u.username ORDER BY total_value DESC LIMIT 100"
  )
  .fetch_all(pool())
  .await?;

  Ok(
    rows
      .into_iter()
      .map(|row| HiscoreEntry {
        username: row.username,
        total_value: row.total_value.unwrap_or_default(),
      })
      .collect(),
  )
}

pub async fn get_user_inventory_count(user_id: i32) -> sqlx::Result<Option<i64>> {
  let count = sqlx::query_scalar!("SELECT COUNT(*) FROM inventory WHERE user_id = $1", user_id)
    .fetch_one(pool())
    .await?;
  Ok(count)
}

pub async fn get_user_storage_upgrade_level(user_id: i32) -> sqlx::Result<i32> {
  sqlx::query_scalar!(
    "SELECT storage_level FROM bases WHERE user_id = $1",
    user_id,
  )
  .fetch_optional(pool())
  .await
  .map(|row| row.unwrap_or(0))
}

pub async fn get_available_inventory_space(user_id: i32) -> sqlx::Result<i32> {
  let item_count = get_user_inventory_count(user_id).await?.unwrap_or(0);

  let inventory_upgrade_level = get_user_storage_upgrade_level(user_id).await?;
  let inventory_capacity =
    BASE_INVENTORY_SIZE as i32 + inventory_upgrade_level * INVENTORY_CAPACITY_PER_UPGRADE as i32;

  Ok(inventory_capacity - item_count as i32)
}

/// Locks a user's inventory for use in a transaction.  Returns all items in the user's inventory.
pub async fn lock_user_inventory(
  txn: &mut sqlx::Transaction<'_, Postgres>,
  user_id: i32,
) -> sqlx::Result<Vec<DbItem>> {
  sqlx::query_as!(
    DbItem,
    "SELECT id, item_id, quality, value, modifiers FROM inventory WHERE user_id = $1 FOR UPDATE",
    user_id
  )
  .fetch_all(&mut **txn)
  .await
}

pub async fn debit_user_inventory(
  txn: &mut sqlx::Transaction<'_, Postgres>,
  user_id: i32,
  debits: &[ItemCost],
) -> Result<(), Status> {
  let inventory = lock_user_inventory(txn, user_id).await.map_err(|err| {
    error!("Failed to lock user inventory: {err}");
    Status::internal("Internal DB error")
  })?;
  let mut items_by_id: FxHashMap<i32, Vec<DbItem>> =
    inventory
      .into_iter()
      .fold(FxHashMap::default(), |mut map, item| {
        map.entry(item.item_id).or_insert_with(Vec::new).push(item);
        map
      });

  // We debit items from lowest quality to highest, so we sort the items from highest to lowest
  // quality
  for items in items_by_id.values_mut() {
    items.sort_unstable_by(|a, b| b.quality.partial_cmp(&a.quality).unwrap());
  }

  let mut item_ids_to_delete = Vec::new();
  for debit in debits {
    let item = items_by_id.get_mut(&(debit.item_id as _)).ok_or_else(|| {
      Status::not_found(format!(
        "Item {:?} not found in inventory",
        get_item_display_name_by_id(debit.item_id as u32)
      ))
    })?;
    let mut remaining_quality = debit.total_quality;

    // Pop items from the back of the list - taking the lowest quality items first - until we've
    // debited the total quality
    while let Some(item) = item.pop() {
      item_ids_to_delete.push(item.id);
      remaining_quality -= item.quality;
      if remaining_quality <= 0.0 {
        break;
      }
    }

    if remaining_quality > 0.0 {
      return Err(Status::resource_exhausted(format!(
        "Not enough quality in inventory for item {:?}; missing {} total quality",
        get_item_display_name_by_id(debit.item_id as u32),
        remaining_quality
      )));
    }
  }

  // Delete the items that were debited
  sqlx::query!(
    "DELETE FROM inventory WHERE id = ANY($1::uuid[])",
    &item_ids_to_delete
  )
  .execute(&mut **txn)
  .await
  .map_err(|err| {
    error!("Failed to delete debited items: {err}");
    Status::internal("Internal DB error")
  })?;

  Ok(())
}

pub(crate) async fn get_user_upgrades(user_id: i32) -> sqlx::Result<Upgrades> {
  let storage_upgrade_level = sqlx::query_scalar!(
    "SELECT storage_level FROM bases WHERE user_id = $1",
    user_id,
  )
  .fetch_optional(pool())
  .await?;

  let total_inventory_capacity = BASE_INVENTORY_SIZE as i32
    + storage_upgrade_level.unwrap_or(0) * INVENTORY_CAPACITY_PER_UPGRADE as i32;
  let storage_level = storage_upgrade_level.unwrap_or(0) as _;

  Ok(Upgrades {
    storage_upgrades: Some(StorageUpgrades {
      storage_capacity: total_inventory_capacity as _,
      storage_level,
      upgrade_cost: get_inventory_upgrade_cost(storage_level).to_vec(),
    }),
  })
}
