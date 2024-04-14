use foundations::telemetry::metrics::{metrics, Counter, Gauge, HistogramBuilder, TimeHistogram};

#[metrics]
pub mod game {
  pub fn active_mine_sessions(location_name: &'static str) -> Gauge;

  pub fn items_mined(location_name: &'static str) -> Counter;

  pub fn item_value_mined(location_name: &'static str) -> Counter;
}

#[metrics]
pub mod db {
  #[ctor = HistogramBuilder {
    buckets: &[0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 1.0, 2.5, 5.0, 10.0]
  }]
  pub fn get_hiscores_duration() -> TimeHistogram;

  #[ctor = HistogramBuilder {
    buckets: &[0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 1.0, 2.5, 5.0, 10.0]
  }]
  pub fn get_user_inventory_duration() -> TimeHistogram;

  #[ctor = HistogramBuilder {
    buckets: &[0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 1.0, 2.5, 5.0, 10.0]
  }]
  pub fn get_user_aggregated_inventory_duration() -> TimeHistogram;

  #[ctor = HistogramBuilder {
    buckets: &[0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 1.0, 2.5, 5.0, 10.0]
  }]
  pub fn get_user_inventory_count_duration() -> TimeHistogram;
}
