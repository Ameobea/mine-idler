use std::f32::consts::E;

use foundations::BootstrapResult;
use fxhash::FxHashMap;
use once_cell::sync::OnceCell;
use rand::{seq::SliceRandom, Rng};
use rand_distr::Distribution;
use scrypt::password_hash::rand_core::RngCore;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
  db::insert_item_descriptors,
  protos::{Item, ItemDescriptor, ItemModifier, MineLocationDescriptor},
};

static ITEM_DESCRIPTORS: OnceCell<Vec<ItemDescriptor>> = OnceCell::new();
static ITEM_DESCRIPTOR_BY_ID: OnceCell<FxHashMap<u32, ItemDescriptor>> = OnceCell::new();
static ITEM_ID_BY_NAME: OnceCell<FxHashMap<String, u32>> = OnceCell::new();

pub fn item_descriptors() -> &'static Vec<ItemDescriptor> {
  ITEM_DESCRIPTORS
    .get()
    .expect("Item descriptors not initialized")
}

pub fn get_item_id_by_name(name: &str) -> u32 {
  ITEM_ID_BY_NAME
    .get()
    .expect("Item ID by name not initialized")
    .get(name)
    .copied()
    .unwrap_or_else(|| panic!("Item with name {name} not found"))
}

fn get_item_descriptor_by_id(id: u32) -> &'static ItemDescriptor {
  ITEM_DESCRIPTOR_BY_ID
    .get()
    .expect("Item descriptor by ID not initialized")
    .get(&id)
    .unwrap_or_else(|| panic!("Item with ID {id} not found"))
}

pub async fn populate_items_table() -> BootstrapResult<()> {
  let item_tables = [include_str!("item_tables/loot.yml")];

  let mut all_item_descriptors = Vec::new();
  for table in item_tables {
    let items: Vec<ItemDescriptor> = serde_yaml::from_str::<Vec<ItemDescriptor>>(table)?;
    insert_item_descriptors(&items).await?;
    all_item_descriptors.extend(items);
  }

  let item_id_by_name = all_item_descriptors
    .iter()
    .map(|item| (item.name.clone(), item.id))
    .collect();
  ITEM_ID_BY_NAME
    .set(item_id_by_name)
    .map_err(|_| anyhow::anyhow!("Item ID by name already initialized"))?;

  let item_descriptor_by_id = all_item_descriptors
    .iter()
    .map(|item| (item.id, item.clone()))
    .collect();
  ITEM_DESCRIPTOR_BY_ID
    .set(item_descriptor_by_id)
    .map_err(|_| anyhow::anyhow!("Item descriptor by ID already initialized"))?;

  ITEM_DESCRIPTORS
    .set(all_item_descriptors)
    .map_err(|_| anyhow::anyhow!("Item descriptors already initialized"))?;

  info!("Successfully populated items table");

  Ok(())
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum QualityDistribution {
  #[default]
  Uniform,
  Normal {
    mean: f32,
    std_dev: f32,
  },
}

impl QualityDistribution {
  /// Generates a quality value from 0.0 to 1.0 based on the distribution.
  pub fn gen(&self, rng: &mut impl RngCore) -> f32 {
    match self {
      QualityDistribution::Uniform => rng.gen_range(0.0..1.0),
      QualityDistribution::Normal { mean, std_dev } => {
        let mut sample = 0.0f32;
        while sample <= 0.0 || sample >= 1.0 {
          let normal = rand_distr::Normal::new(*mean, *std_dev).unwrap();
          sample = normal.sample(rng);
        }
        sample
      },
    }
  }
}

#[derive(Serialize)]
pub struct LootTableItemEntry {
  pub weight: f32,
  pub id: u32,
  #[serde(default)]
  pub quality_distribution: QualityDistribution,
}

// Custom impl because the `id` field isn't actually in the struct.  Instead, there's a `name` field
// which we need to look up in `ITEM_ID_BY_NAME`.
impl<'de> Deserialize<'de> for LootTableItemEntry {
  fn deserialize<D>(deserializer: D) -> Result<LootTableItemEntry, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    struct LootTableItemEntryHelper {
      weight: f32,
      name: String,
      #[serde(default)]
      quality_distribution: QualityDistribution,
    }

    let helper = LootTableItemEntryHelper::deserialize(deserializer)?;
    let id = get_item_id_by_name(&helper.name);

    Ok(LootTableItemEntry {
      weight: helper.weight,
      id,
      quality_distribution: helper.quality_distribution,
    })
  }
}

// https://i.ameo.link/c1b.png
fn compute_item_quality_multiplier(quality: f32) -> f32 {
  let k1: f32 = 18.;
  let k2: f32 = 12.;
  let m1: f32 = 0.02;
  let m2: f32 = 0.98;

  let exp_k1_x_m1 = E.powf(k1 * (quality - m1));
  let exp_neg_k2_m2_x = E.powf(-k2 * (m2 - quality));

  let val = exp_k1_x_m1 / (1.0 + exp_k1_x_m1) + exp_neg_k2_m2_x / (1.0 + exp_neg_k2_m2_x);
  let val = (val.powf(1.6) - 0.18) * 0.8
    + quality.powf(1.6) * 0.2
    + quality.powf(3.5) * 0.15
    + quality.powf(10.0) * 1.8;

  val * 1.2
}

fn compute_item_value(id: u32, quality: f32, _modifiers: &[ItemModifier]) -> f32 {
  let item_descriptor = get_item_descriptor_by_id(id);
  let base_value = match item_descriptor.rarity_tier {
    0 => 0.2,
    1 => 4.,
    2 => 30.0,
    3 => 230.0,
    4 => 800.0,
    5 => 8_400.,
    _ => 0.0,
  };

  let quality_multiplier = compute_item_quality_multiplier(quality);

  // TODO: Implement modifier value calculation

  base_value * quality_multiplier
}

impl LootTableItemEntry {
  pub fn gen(&self, rng: &mut impl RngCore) -> Item {
    let quality = self.quality_distribution.gen(rng);
    let modifiers = self.gen_modifiers(rng);
    let value = compute_item_value(self.id, quality, &modifiers);

    Item {
      id: self.id as i32,
      modifiers,
      quality,
      value,
    }
  }

  fn gen_modifiers(&self, _rng: &mut impl RngCore) -> Vec<ItemModifier> { Vec::new() }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum LootTableEntry {
  #[serde(rename = "item")]
  Item(LootTableItemEntry),
  #[serde(rename = "subtable")]
  Subtable { weight: f32, table: Box<LootTable> },
}

impl LootTableEntry {
  pub fn weight(&self) -> f32 {
    match self {
      LootTableEntry::Item(entry) => entry.weight,
      LootTableEntry::Subtable { weight, .. } => *weight,
    }
  }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct LootTable(Vec<LootTableEntry>);

impl LootTable {
  pub fn roll(&self, rng: &mut impl RngCore) -> Item {
    let choice = self.0.choose_weighted(rng, |entry| entry.weight()).unwrap();
    match choice {
      LootTableEntry::Item(entry) => entry.gen(rng),
      LootTableEntry::Subtable { table, .. } => table.roll(rng),
    }
  }
}

pub struct MineLocation {
  pub descriptor: MineLocationDescriptor,
  pub loot_table: LootTable,
}

static MINE_LOCATIONS: OnceCell<Vec<MineLocation>> = OnceCell::new();

pub fn mine_locations() -> &'static Vec<MineLocation> {
  MINE_LOCATIONS
    .get()
    .expect("Mine locations not initialized")
}

pub fn init_loot_tables() -> BootstrapResult<()> {
  let location_descriptors: Vec<MineLocationDescriptor> =
    serde_yaml::from_str(include_str!("mine_locations.yml"))?;

  let loot_tables = [("starter", include_str!("loot_tables/starter.yml"))];

  let mut locations = Vec::new();
  for descriptor in location_descriptors {
    let loot_table = loot_tables
      .iter()
      .find(|(name, _)| name == &descriptor.name)
      .map(|(_, table)| serde_yaml::from_str::<LootTable>(table).unwrap())
      .ok_or_else(|| anyhow::anyhow!("No loot table found for location {}", descriptor.name))?;
    locations.push(MineLocation {
      descriptor,
      loot_table,
    });
  }

  MINE_LOCATIONS
    .set(locations)
    .map_err(|_| anyhow::anyhow!("Mine locations already initialized"))?;

  info!("Initialized loot tables");

  Ok(())
}

#[test]
fn loot_table_serialize() {
  let table = LootTable(vec![
    LootTableEntry::Item(LootTableItemEntry {
      id: 1,
      weight: 1.0,
      quality_distribution: QualityDistribution::Uniform,
    }),
    LootTableEntry::Subtable {
      table: Box::new(LootTable(vec![LootTableEntry::Item(LootTableItemEntry {
        id: 2,
        weight: 1.0,
        quality_distribution: QualityDistribution::Normal {
          mean: 0.2,
          std_dev: 0.1,
        },
      })])),
      weight: 1.0,
    },
  ]);

  // write to /tmp/loot_table.yml
  let table_str = serde_yaml::to_string(&table).unwrap();
  std::fs::write("/tmp/loot_table.yml", table_str).unwrap();
}
