use crate::WindData;
use circular_queue::CircularQueue;
use tabular::{Row, Table};

pub fn moving_avg_main(wind_data: &WindData) {
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
            window.push(*total, date);
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
    println!();
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
