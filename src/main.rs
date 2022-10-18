//mod with_storage;

use circular_queue::CircularQueue;
use std::io;
use tabular::{Row, Table};

fn main() -> io::Result<()> {
    // load the wind data using the given files
    let wind_data = load_wind_data(
        "./ninja_europe_wind_v1.1/capacity_current_national.csv",
        "./ninja_europe_wind_v1.1/ninja_wind_europe_v1.1_current_national.csv",
    )?;

    // Create the initial windows, each for a different number of hours
    let mut windows = [
        1,
        12,
        1 * 24,
        5 * 24,
        10 * 24,
        25 * 24,
        50 * 24,
        100 * 24,
        200 * 24,
    ]
    .into_iter()
    .map(|hours| Window::new(hours))
    .collect::<Vec<_>>();

    // push all the data into the windows to calculate the moving averages for each window size
    for (date, total) in wind_data {
        for window in &mut windows {
            window.push(total, &date);
        }
    }

    // format the output as a table
    let mut table = Table::new("{:<}   {:<}   {:<}   {:<}   {:<}");
    table.add_row(
        Row::new()
            .with_cell("Window Size")
            .with_cell("Min (GW)")
            .with_cell("Max (GW)")
            .with_cell("Min as % Avg")
            .with_cell("Date of Min"),
    );
    table.add_heading("");
    let average = windows[0].average_of_all_values();
    for window in windows {
        let size_text = if window.window_size_hours() < 24 {
            format!("{: >3} hrs", window.window_size_hours())
        } else {
            format!("{: >3} days ", window.window_size_hours() as f64 / 24.0)
        };
        let row = Row::new()
            .with_cell(size_text)
            .with_cell(format!("{:.2}", window.min_moving_average))
            .with_cell(format!("{:.2}", window.max_moving_average))
            .with_cell(format!(
                "{:.2}%",
                window.min_moving_average / window.average_of_all_values() * 100.0
            ))
            .with_cell(window.min_moving_date.unwrap_or_default());
        table.add_row(row);
    }
    println!("{}", table);
    println!("Average output: {:.2} GW", average);
    Ok(())
}

type WindData = Vec<(String, f64)>;

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
                capacities[index - 1] / 1000.0 * ratio // capacity data is in MW, we want GW
            })
            .sum();
        outputs.push((String::from(record.get(0).unwrap()), total));
    }
    Ok(outputs)
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
        .collect())
}

/// Represents a window for calculating a moving average
struct Window {
    /// A circular queue holding the values to average
    queue: CircularQueue<f64>,
    /// the total of all values put into the window
    total_of_all_samples: f64,
    /// the total number of samples stored into the window
    sample_count: usize,
    /// the minimum value of the moving average that has been seen
    min_moving_average: f64,
    /// the maximum value of the moving average that has been seen
    max_moving_average: f64,
    /// the date at which the minimum moving average was recorded
    min_moving_date: Option<String>,
}

impl Window {
    fn new(num_hours: usize) -> Self {
        Window {
            queue: CircularQueue::with_capacity(num_hours),
            min_moving_average: f64::MAX,
            max_moving_average: f64::MIN,
            total_of_all_samples: 0.0,
            sample_count: 0,
            min_moving_date: None,
        }
    }

    fn push(&mut self, value: f64, date: &str) {
        // write another value into the window, potentially pushing out a value that's too old
        self.queue.push(value);

        // if we've got a full buffer of data points ...
        if self.queue.len() >= self.queue.capacity() {
            // calculate the current moving average and update the min/max that we've seen
            let average = self.current_moving_average();
            self.max_moving_average = self.max_moving_average.max(average);
            if average < self.min_moving_average {
                // record the date that this minimum was seen
                self.min_moving_date = Some(String::from(date));
            }
            self.min_moving_average = self.min_moving_average.min(average);
        }

        // update the values used to calculate the average of all values
        self.total_of_all_samples += value;
        self.sample_count += 1;
    }

    /// Calculates the current value of the moving average based on the values in the queue
    fn current_moving_average(&self) -> f64 {
        self.queue.iter().sum::<f64>() / (self.queue.len() as f64)
    }

    /// Returns the window size of the `Window` in hours
    fn window_size_hours(&self) -> usize {
        self.queue.capacity()
    }

    /// Returns the average of all values put into the `Window`
    fn average_of_all_values(&self) -> f64 {
        self.total_of_all_samples / (self.sample_count as f64)
    }
}
