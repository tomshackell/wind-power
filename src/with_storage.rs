use crate::WindData;
use tabular::{row, Table};

struct Storage {
    capacity: f64,
    current: f64,
    max_shortfall: f64,
    max_shortfall_date: Option<String>,
}

impl Storage {
    fn new(capacity_gwh: f64) -> Self {
        Self {
            capacity: capacity_gwh,
            current: capacity_gwh, // storage starts full: simplifies the analysis
            max_shortfall: 0.0,
            max_shortfall_date: None,
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

/// The amount of demand required (in GW)
const DEMAND_LOAD: f64 = 29.96;

pub fn storage_main(wind_data: &WindData) {
    let storage_amounts = [
        0.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 100_000.0,
    ];
    let mut table = Table::new("{:<}   {:<}   {:<}");
    table.add_row(row!(
        "Storage (GWh)",
        "Overbuild factor required",
        "Backup required (GW)"
    ));
    table.add_heading("");
    for storage in storage_amounts {
        let required_overbuild = find_required_overbuild(storage, &wind_data);
        let required_backup = find_required_backup(storage, &wind_data);
        table.add_row(row!(
            storage,
            format!("{:.2}", required_overbuild),
            format!("{:.2}", required_backup)
        ));
    }
    println!("{}", table);
}

/// Find the amount of backup that is required to ensure demand is always met.
fn find_required_backup(storage_gwh: f64, wind_data: &WindData) -> f64 {
    let mut storage = Storage::new(storage_gwh);
    for (date, total) in wind_data {
        let extra = total - DEMAND_LOAD;
        storage.add(extra, date);
    }
    storage.max_shortfall
}

/// Find the amount of overbuild required to ensure there is never a shortage, given a certain
/// amount of `storage_gwh` and `wind_data`
fn find_required_overbuild(storage_gwh: f64, wind_data: &WindData) -> f64 {
    // this works using binary search to find the right level of overbuild (to within 0.001)
    let mut lower_limit = 0.0;
    let mut upper_limit = 1000.0; // we assume overbuild by 1000 is always enough
    while (upper_limit - lower_limit) > 0.001 {
        let mid = (upper_limit + lower_limit) / 2.0;
        if always_meets_demand(storage_gwh, wind_data, mid) {
            upper_limit = mid; // then we know mid is sufficient, move upper limit down
        } else {
            lower_limit = mid; // then we know mid is insufficient, move lower limit up
        }
    }
    (upper_limit + lower_limit) / 2.0
}

/// Returns whether the given configuration of `storage_gwh`, `wind_data` and `overbuild` will
/// always managed to meet the required amount of demand.
fn always_meets_demand(storage_gwh: f64, wind_data: &WindData, overbuild: f64) -> bool {
    let mut storage = Storage::new(storage_gwh);
    for (date, total) in wind_data {
        let extra = total * overbuild - DEMAND_LOAD;
        if storage.add(extra, date) {
            return false;
        }
    }
    true
}
