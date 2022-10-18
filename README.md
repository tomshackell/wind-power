# Wind power analysis

This analyses the [Ninja v1.1 wind power data](https://www.renewables.ninja/downloads).

## How to run
1. Install rust using [rustup](https://rustup.rs/)
3. Download the [Ninja v1.1 wind power data](https://www.renewables.ninja/static/downloads/ninja_europe_wind_v1.1.zip) and unpack so the `./ninja_europe_wind_v1.1/ninja_wind_europe_v1.1_current_national.csv` exists (relative to this README file). 
2. Run `cargo run --release` to run the analysis

## Analysis of results

The nature of the analysis is:
- calculate the total power (GWs) generated across all of the countries in the model using the capacity ratios in `ninja_wind_europe_v1.1_current_national.csv` and the listed installed capacity per country in the associated metadata and taking the sum across each row.
- then track the [moving average](https://en.wikipedia.org/wiki/Moving_average) of that total GW output value for different window sizes
- then record the minimum/maximum of that moving average across the entire data set for each window size.

Here are the results:
```
Window Size   Min (GW)   Max (GW)   Min as % Avg   Date of Min

  1 hrs       2.71       95.33      9.06%          2009-07-01 06:00:00
 12 hrs       4.57       91.56      15.24%         2007-06-11 16:00:00
  1 days      7.04       88.16      23.50%         2007-06-12 00:00:00
  5 days      9.19       74.08      30.67%         1982-05-21 21:00:00
 10 days      11.33      67.37      37.83%         2006-07-30 04:00:00
 25 days      14.35      58.32      47.88%         1997-08-26 13:00:00
 50 days      16.12      55.48      53.80%         2006-07-31 03:00:00
100 days      19.33      48.63      64.51%         2003-09-03 12:00:00
200 days      21.86      40.74      72.97%         2014-10-15 21:00:00

Average output: 29.96 GW
```
Meaning of the columns:
- `Window size` the size of window used to calculate the moving average
- `Min (GW)` the minimum moving average of the amount of power generated across all 30 countries in the data set in total.
- `Max (GW)` the maximum moving average of the amount of power generated across all 30 countries in the data set in total.
- `Min as % Avg` what the minimum is as a percentage of the average output. The average output for all 30 countries is measured at 29.96 GW, this tallies exactly with the installed capacity (119.4 GW) and the expected capacity factor (25.1%) from the metadata, (119.4 * 0.251 = 29.96)
- `Date of Min` when that minimum occurred in the data set.

So for a 1 hour moving average, which corresponds to having no storage:
- Summing all of the power generated across all 30 countries the total generation was at a minimum at 2009-07-01 06:00:00 with a total generation of just 2.71 GW. This is just 9.06% of the average output (29.96 GW).
- So to *reliably* produce 29.96 GW you would either need 27.25 GW (29.96 - 2.71) of backup capacity ready to take over.
- OR you would need to over build the wind turbines by a factor of 11 (1/0.0906)

For a 5 day moving average:
- The lowest combined 5-day moving average output summed across all 30 countries was 9.19 GW, which is 30.67% of the average output (29.96 GW) this happened at 1982-05-21 21:00:00.
- To reliably produce 29.96 GW you would either need 20.77 GW (29.96 - 9.19) of backup capacity ready to take over.
- OR you would need to over build the wind turbines by a factor of 3.2 (1/0.3067)
- You would also need to produce significant storage: enough to cover any reduction in output less than this 5 day minimum, which is (ballpark) around 5 days at 20.77 GW, totalling 2,492.4 GWh (5 * 24 * 20.77) of storage. For context, this would be more than [7 times the amount of pumped storage Europe currently has](https://www.dnv.com/news/estorage-study-shows-huge-potential-capacity-of-exploitable-pumped-hydro-energy-storage-sites-in-europe-63675).
- Also this is even though we're only looking at 29.96 GW of production and Europe's total electricity need is 10 times that figure ([roughly 300 GW](https://ec.europa.eu/eurostat/statistics-explained/index.php?title=Electricity_production,_consumption_and_market_overview)) and it's total energy need more like 40 times that figure.


Similarly for a 25 day moving average:
- You would either need 15.61 GW of backup capacity
- OR you would need to over-build by roughly a factor of 2 (1/0.4788)
- You also would need enough storage to cover the deficit for any lull shorter than this 25 day minimum point, roughly 25 days at 15.61GW, or 9,366 GWh. Again, for context this would be more pumped storage than [exists in the entire world](https://www.hydropower.org/factsheets/pumped-storage) and we're still only talking about meeting 10% of Europe's electricity needs and perhaps 3% of its total energy needs.

In conclusion:
- Periods of highly depressed wind output across all of Europe are common place, there are many examples in the data set. This is true both at short term frames (1 hour) and long time frames (25 days).
- This means for wind to provide reliable output it must either a) have large amounts of storage, b) massively overbuild, or c) have significant on-demand backup capacity available.
- You can trade off somewhat between these three options: but any of them is going to be expensive.
- The effect of storage is somewhat complex and I may do some more in-depth analysis of how adding different amount of storage changes the amount of over-build/backup required: the figures for storage above for 5-day and 25-day are an estimate.
- The cheapest option (by far) is to have backup capacity available on standby: this is what Germany has done in practice. With current available technology this backup must either by hydropower, which is limited in scale, or more commonly it is met by fossil fuels (as Germany is doing).
- This analysis has just summed the power across the countries and assumed perfect interconnection is available for free at any capacity that's required: in reality it's not and adding that interconnection would add significantly to the expense.
- As previously stated: VRE can be cheap as a modest part of a flexible grid, but trying to get it to produce reliable electricity is extremely hard and the current LCOE figures would need to be multiplied by several fold to start getting anywhere near this.


