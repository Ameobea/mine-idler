use foundations::telemetry::metrics::{metrics, Counter, Gauge};

#[metrics]
pub mod game {
  pub fn active_mine_sessions(location_name: &'static str) -> Gauge;

  pub fn items_mined(location_name: &'static str) -> Counter;

  pub fn item_value_mined(location_name: &'static str) -> Counter;
}
