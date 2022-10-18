# Wind power analysis

This is a best case analysis for trying to produce reliable power with 100% wind, using the (widely accepted) [Ninja v1.1 wind power data](https://www.renewables.ninja/downloads). The dataset takes the turbines that are installed today and predicts what power they would have produced in the past based on historical wind readings.
- The first part of the analysis is an analysis of the moving average of total European wind power production. It shows there is considerable variation in the amount of wind power even when aggregated at the continent level.
- The second is an analysis of how much storage would be required to produce reliable power using wind.

## How to run
1. Install rust using [rustup](https://rustup.rs/)
3. Download the [Ninja v1.1 wind power data](https://www.renewables.ninja/static/downloads/ninja_europe_wind_v1.1.zip) and unpack so the `./ninja_europe_wind_v1.1/ninja_wind_europe_v1.1_current_national.csv` exists (relative to this README file). 
2. Run `cargo run --release` to run the analysis

## Moving average analysis

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

## Storage analysis

The analysis simulates having a certain amount of storage available.
- It assumes the storage is perfectly efficient (every 1 GWh stored can be released at any time later as 1 GWh).
- It assumes the required demand is a constant 29.96 GW (the average output)
- It simulates the system with the storage. 
  - If wind power is generated in excess of the demand that extra is stored (up to the capacity of the storage) 
  - If insufficient wind power is available to meet the demand then energy is drawn down from the storage as needed. The amount of energy stored will never go negative but instead there can be a shortfall in matching the demand if not enough energy is stored.  
- For a given amount of storage it tracks:
  - How much overbuild would be required to ensure supply always meets demand.  
    - This is found using [binary search](https://en.wikipedia.org/wiki/Binary_search_algorithm).
    - It assumes we can simulate overbuild by multiplying the actual historical values by a constant factor. This is not a perfect estimate (distribution matters), but serves as a good enough approximation for these purposes. Given there are now wind turbines all across Europe it is reasonable to estimate that if you built twice as many wind turbines you would get roughly twice as much power as you do today. 
  - Or without any overbuild how much backup capacity would be required to ensure supply always meets demand. 
    - This is found by noting what the largest shortfall seen is when trying to draw power from the storage.
  
The results:
```
Storage (GWh)   Overbuild factor required   Backup required (GW)

0               11.04                       27.25
10              9.01                        27.25
20              7.98                        27.25
50              6.14                        27.25
100             4.74                        27.25
200             3.43                        27.25
500             2.88                        27.25
1000            2.40                        27.25
2000            1.92                        27.25
5000            1.62                        27.25
10000           1.40                        27.25
150000          0.99                        0.00

```
Meaning of the columns:
- `Storage (GWh)` - the amount of storage available in GWh
- `Overbuild factor required`- given this amount of storage, and the wind data, how much overbuild is required to ensure that the demand (constant 29.96GW) is always met without any kind of backup capacity. 
- `Backup required (GW)` - without any overbuild how much backup capacity is required to ensure the demand is always met.

Analysis:
- With no storage you must either overbuild by a factor of 11.04, or provide 27.25 GW of backup.
- With 10 GWh of storage (NOTE: [all the storage batteries in the world in 2018 amounted to 8 GWh](https://www.worldenergy.org/assets/downloads/ESM_Final_Report_05-Nov-2019.pdf)) you must overbuild by a factor of 9.01 or alternatively provide 27.25 GW of backup.
- With 500 GWh of storage (NOTE: this is [more storage than all the pumped storage in Europe](https://www.dnv.com/news/estorage-study-shows-huge-potential-capacity-of-exploitable-pumped-hydro-energy-storage-sites-in-europe-63675)) you would need to overbuild by a factor of 2.88.
- With 10,000 GWh of storage (NOTE: this is [more storage than all the pumped storage in the world](https://www.hydropower.org/factsheets/pumped-storage)) you would need to overbuild by a factor of 1.40.
- With 150,000 GWh of storage no overbuild or backup capacity is required to provide the reliable 29.96 GW.

This is to produce 29.96GW of reliable power. For comparison Europe's total electricity need is roughly 10x this figure (around 300GW) and it's total energy need is roughly 40 times this figure. The amount of storage required to produce 40x the amount of power would be 40x as much according to this simulation. 

## Conclusions 

In conclusion:
- Periods of highly depressed wind output across all of Europe are common place, there are many examples in the data set. This is true both at short term frames (1 hour) and long time frames (25 days).
- This means for wind to provide reliable output it must either a) have large amounts of storage, b) massively overbuild, or c) have significant on-demand backup capacity available.
- You can trade off somewhat between these three options: but any of them is going to be expensive.
- The cheapest option (by far) is to have backup capacity available on standby: this is what Germany has done in practice. With current available technology this backup must either by hydropower, which is limited in scale, or more commonly it is met by fossil fuels (as Germany is doing).
- This analysis has just summed the power across the countries and assumed perfect interconnection is available for free at any capacity that's required: in reality it's not and adding that interconnection would add significantly to the expense.
- As previously stated: VRE can be cheap as a modest part of a flexible grid, but trying to get it to produce reliable electricity is extremely hard and the current LCOE figures would need to be multiplied by several fold to start getting anywhere near this.


