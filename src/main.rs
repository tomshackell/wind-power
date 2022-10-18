//mod with_storage;

use circular_queue::CircularQueue;
use std::io;
use tabular::{Row, Table};

fn main() -> io::Result<()> {
    let wind_data = load_wind_data(
        "./ninja_europe_wind_v1.1/capacity_current_national.csv",
        "./ninja_europe_wind_v1.1/ninja_wind_europe_v1.1_current_national.csv",
    )?;

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

    for (date, total) in wind_data {
        for window in &mut windows {
            window.push(total, &date);
        }
    }
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
    let average = windows[0].average();
    for window in windows {
        let size_text = if window.window_size_hours() < 24 {
            format!("{: >3} hrs", window.window_size_hours())
        } else {
            format!("{: >3} days ", window.window_size_hours() as f64 / 24.0)
        };
        let row = Row::new()
            .with_cell(size_text)
            .with_cell(format!("{:.2}", window.min_average))
            .with_cell(format!("{:.2}", window.max_average))
            .with_cell(format!(
                "{:.2}%",
                window.min_average / window.average() * 100.0
            ))
            .with_cell(window.min_date.unwrap_or_default());
        table.add_row(row);
    }
    println!("{}", table);
    println!("Average output: {:.2} GW", average);
    Ok(())
}

type WindData = Vec<(String, f64)>;

fn load_wind_data(capacities_file: &str, data_file: &str) -> io::Result<WindData> {
    let capacities = read_capacities(capacities_file)?;
    let mut data_rdr = csv::Reader::from_path(data_file)?;
    let mut outputs = Vec::new();
    for result in data_rdr.records() {
        let record = result?;
        let total: f64 = (1..record.len())
            .map(|index| {
                let string = record.get(index).unwrap();
                let ratio = str::parse::<f64>(string).expect("Should be valid float");
                capacities[index - 1] / 1000.0 * ratio
            })
            .sum();
        let date = String::from(record.get(0).unwrap());
        outputs.push((date, total));
    }
    Ok(outputs)
}

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

struct Window {
    queue: CircularQueue<f64>,
    min_average: f64,
    max_average: f64,
    total: f64,
    sample_count: usize,
    min_date: Option<String>,
}

impl Window {
    fn new(num_hours: usize) -> Self {
        Window {
            queue: CircularQueue::with_capacity(num_hours),
            min_average: f64::MAX,
            max_average: f64::MIN,
            total: 0.0,
            sample_count: 0,
            min_date: None,
        }
    }

    fn push(&mut self, value: f64, date: &str) {
        self.queue.push(value);
        let average = self.current_moving_average();
        self.max_average = self.max_average.max(average);
        // only if we've got a full buffer of data points
        if self.queue.len() >= self.queue.capacity() {
            if average < self.min_average {
                self.min_date = Some(String::from(date));
            }
            self.min_average = self.min_average.min(average);
        }
        self.total += value;
        self.sample_count += 1;
    }

    fn current_moving_average(&self) -> f64 {
        self.queue.iter().sum::<f64>() / (self.queue.len() as f64)
    }

    fn window_size_hours(&self) -> usize {
        self.queue.capacity()
    }

    fn average(&self) -> f64 {
        self.total / (self.sample_count as f64)
    }
}
