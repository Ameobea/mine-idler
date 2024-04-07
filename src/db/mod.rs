use std::time::Duration;

use foundations::BootstrapResult;
use once_cell::sync::OnceCell;
use sqlx::{
  pool::PoolOptions,
  postgres::{PgConnectOptions, PgQueryResult},
  FromRow, Pool, Postgres,
};
use tonic::Status;

use crate::{
  auth::hash_password,
  conf::Settings,
  game::items::populate_items_table,
  protos::{Item, ItemDescriptor, SortBy, SortDirection, UserAccountInfo},
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

fn pool() -> &'static Pool<Postgres> { DB_POOL.get().expect("Database pool not initialized") }

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

  #[derive(FromRow)]
  struct DbItem {
    item_id: i32,
    quality: f32,
    value: f32,
    modifiers: Option<serde_json::Value>,
  }

  let items: Vec<DbItem> = sqlx::query_as(&format!(
    "SELECT inv.item_id, inv.quality, inv.value, inv.modifiers FROM inventory inv JOIN items i ON \
     inv.item_id = i.id WHERE inv.user_id = $1 ORDER BY {sort_column} {sort_direction} LIMIT $2 \
     OFFSET $3"
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
