use crate::WindData;
use tabular::{row, Table};

struct Storage {
    /// Total capacity of the storage (in GWh)
    capacity: f64,
    /// The current amount of energy stored in the storage (in GWh)
    current: f64,
    /// The maximum shortfall that was seen when trying to draw from the storage (in GWh)
    max_shortfall: f64,
    /// The date at which that shortfall occurred
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

    /// Adds `delta` extra GWh to the storage, which can be negative to draw it down. Returns
    /// `true` if `delta` was positive or there was enough in storage to provide the delta and
    /// `false` if `delta` was negative and there was not enough in storage to meet it.   
    fn add(&mut self, delta: f64, date: &str) -> bool {
        let new_current = (self.current + delta).min(self.capacity);
        self.current = new_current.max(0.0);

        // calculate the amount of shortfall (if any)
        let shortfall = self.current - new_current;
        if shortfall > self.max_shortfall {
            self.max_shortfall = shortfall;
            self.max_shortfall_date = Some(String::from(date));
        }
        shortfall <= 0.0 // true if there was no shortfall, or false if delta could not be met
    }
}

pub fn storage_main(wind_data: &WindData, demand: f64) {
    let average_output =
        wind_data.iter().map(|(_, output)| output).sum::<f64>() / (wind_data.len() as f64);
    let wind = WindPower {
        data: wind_data,
        /// We scale the amount of wind power so the average output matches the demand
        scaling_factor: demand / average_output,
        demand,
    };
    let storage_amounts = [
        0.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10_000.0, 20_000.0, 50_000.0, 100_000.0,
    ];
    let mut table = Table::new("{:<}   {:<}   {:<}   {:<}");
    table.add_row(row!(
        "Storage (GWh)",
        "Overbuild factor required",
        "Backup required (GW)",
        "Backup required 2x overbuild (GW)",
    ));
    table.add_heading("");
    for storage in storage_amounts {
        let required_overbuild = find_required_overbuild(storage, &wind);
        let required_backup = find_required_backup(storage, &wind, 1.0);
        let required_backup2 = find_required_backup(storage, &wind, 2.0);
        table.add_row(row!(
            storage,
            format!("{:.2}", required_overbuild),
            format!("{:.2}", required_backup),
            format!("{:.2}", required_backup2),
        ));
    }
    println!("{}", table);
    println!(
        "Storage required for no overbuild (GWh): {:.1}",
        find_using_bisection(0.0, f64::MAX, |storage| always_meets_demand(
            storage, &wind, 1.0
        ))
    );
}

struct WindPower<'a> {
    data: &'a WindData,
    scaling_factor: f64,
    demand: f64,
}

impl WindPower<'_> {
    fn outputs(&self) -> impl Iterator<Item = (&str, f64)> {
        self.data
            .iter()
            .map(|(date, output)| (date.as_str(), *output * self.scaling_factor))
    }
}

/// Find the amount of backup that is required to ensure demand is always met.
fn find_required_backup(storage_gwh: f64, wind: &WindPower, overbuild: f64) -> f64 {
    let mut storage = Storage::new(storage_gwh);
    for (date, wind_gw) in wind.outputs() {
        let extra_gw = wind_gw * overbuild - wind.demand;
        storage.add(extra_gw, date);
    }
    storage.max_shortfall
}

/// Find the amount of overbuild required to ensure there is never a shortage, given a certain
/// amount of `storage_gwh` and `wind_data`
fn find_required_overbuild(storage_gwh: f64, wind: &WindPower) -> f64 {
    find_using_bisection(0.0, 1000.0, |overbuild| {
        always_meets_demand(storage_gwh, wind, overbuild)
    })
}

fn find_using_bisection(low: f64, high: f64, test: impl Fn(f64) -> bool) -> f64 {
    assert!(low < high);
    let mut lower_limit = low;
    let mut upper_limit = high;
    while (upper_limit - lower_limit) > 0.001 {
        let mid = (upper_limit + lower_limit) / 2.0;
        if test(mid) {
            upper_limit = mid;
        } else {
            lower_limit = mid;
        }
    }
    (upper_limit + lower_limit) / 2.0
}

/// Returns whether the given configuration of `storage_gwh`, `wind_data` and `overbuild` will
/// always managed to meet the required amount of demand.
fn always_meets_demand(storage_gwh: f64, wind: &WindPower, overbuild: f64) -> bool {
    let mut storage = Storage::new(storage_gwh);
    for (date, wind_gw) in wind.outputs() {
        let extra_gw = wind_gw * overbuild - wind.demand;
        if !storage.add(extra_gw, date) {
            return false;
        }
    }
    true
}
