use crate::{load_wind_data, WindData};
use std::io;

struct Storage {
    capacity: f64,
    current: f64,
    max_shortfall: f64,
    max_shortfall_date: Option<String>,
    cumulative_shortfall: f64,
}

impl Storage {
    fn new(capacity_gwh: f64) -> Self {
        Self {
            capacity: capacity_gwh,
            current: capacity_gwh,
            max_shortfall: 0.0,
            max_shortfall_date: None,
            cumulative_shortfall: 0.0,
        }
    }

    fn add(&mut self, delta: f64, date: &str) -> bool {
        let new_current = (self.current + delta).min(self.capacity);
        self.current = new_current.max(0.0);
        let shortfall = self.current - new_current;
        if shortfall > self.max_shortfall {
            self.max_shortfall = shortfall;
            self.max_shortfall_date = Some(String::from(date));
            true
        } else {
            false
        }
    }
}

const DEMAND_LOAD: f64 = 29.96;

pub fn storage_main() -> io::Result<()> {
    let wind_data = load_wind_data(
        "./ninja_europe_wind_v1.1/capacity_current_national.csv",
        "./ninja_europe_wind_v1.1/ninja_wind_europe_v1.1_current_national.csv",
    )?;
    let storage_amounts = [
        0.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 100_000.0,
    ];
    for storage in storage_amounts {
        let required_overbuild = find_required_overbuild(storage, &wind_data);
        let required_backup = find_required_backup(storage, &wind_data);
        println!(
            "{:>8} {: >10.2} {: >10.2}",
            storage, required_overbuild, required_backup
        );
    }
    Ok(())
}

fn find_required_backup(storage_gwh: f64, wind_data: &WindData) -> f64 {
    let mut storage = Storage::new(storage_gwh);
    for (date, total) in wind_data {
        let extra = total - DEMAND_LOAD;
        storage.add(extra, date);
    }
    storage.max_shortfall
}

fn find_required_overbuild(storage_gwh: f64, wind_data: &WindData) -> f64 {
    let mut low = 0.0;
    let mut high = 100.0;
    while (high - low) > 0.01 {
        let mid = (high + low) / 2.0;
        if requires_overbuild(storage_gwh, wind_data, mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    (high + low) / 2.0
}

fn requires_overbuild(storage_gwh: f64, wind_data: &WindData, overbuild: f64) -> bool {
    let mut storage = Storage::new(storage_gwh);
    for (date, total) in wind_data {
        let extra = total * overbuild - DEMAND_LOAD;
        if storage.add(extra, date) {
            return true;
        }
    }
    false
}
