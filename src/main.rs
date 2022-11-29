mod cost;
mod moving_avg;
mod with_storage;

use std::io;

fn main() -> io::Result<()> {
    // load the wind data using the given files
    let wind_data = load_wind_data(
        "./ninja_europe_wind_v1.1/capacity_current_national.csv",
        "./ninja_europe_wind_v1.1/ninja_wind_europe_v1.1_current_national.csv",
    )?;

    moving_avg::moving_avg_main(&wind_data);
    with_storage::storage_main(&wind_data);
    cost::cost_main(&wind_data);
    Ok(())
}

/// (Date-as-string, GW-output)
/// Wind data is hourly
pub struct WindData {
    pub output_gw: Vec<(String, f64)>,
    pub total_capacity_gw: f64,
    pub average_output_gw: f64,
}

/// Load wind data from the file. This gives an output `Vec<(String, f64)>` which is a collection of
/// dates (as a `String`) and the `f64` as the absolute total GW produced across all countries
/// together
fn load_wind_data(capacities_file: &str, data_file: &str) -> io::Result<WindData> {
    let capacities = read_capacities(capacities_file)?;
    let mut data_rdr = csv::Reader::from_path(data_file)?;
    let mut outputs = Vec::new();
    for result in data_rdr.records() {
        let record = result?;
        // calculate total GW output for all countries
        let total: f64 = (1..record.len())
            .map(|index| {
                // the values in the wind data files
                // (e.g. ninja_wind_europe_v1.1_current_national.csv) are expressed as a ratio of
                // maximum capacity. So combine with the capacities data to get an absolute number
                // of GW.
                let string = record.get(index).unwrap();
                let ratio = str::parse::<f64>(string).expect("Should be valid float");
                capacities[index - 1] * ratio
            })
            .sum();
        outputs.push((String::from(record.get(0).unwrap()), total));
    }
    let average_output_gw = outputs.iter().map(|(_, gw)| *gw).sum::<f64>() / (outputs.len() as f64);
    Ok(WindData {
        output_gw: outputs,
        total_capacity_gw: capacities.iter().sum(),
        average_output_gw,
    })
}

/// Read the total wind capacity data. Returns a GW installed capacity per country, in order they
/// are in the capacities file.
fn read_capacities(file: &str) -> io::Result<Vec<f64>> {
    let mut rdr = csv::Reader::from_path(file)?;
    let row = rdr
        .records()
        .into_iter()
        .next()
        .expect("Must have at least one row")?;
    Ok(row
        .iter()
        .map(|string| str::parse::<f64>(string).expect("Should be valid float"))
        .map(|mw| mw / 1000.0) // we want GW
        .collect())
}
