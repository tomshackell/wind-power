# Wind power analysis

This is a best case analysis for trying to produce reliable power with 100% wind, using the [Ninja v1.1 wind power data](https://www.renewables.ninja/downloads). The dataset takes the turbines that are installed today and predicts what power they would have produced in the past based on historical wind readings.
- The first part of the analysis is an analysis of the moving average of total European wind power production. It shows there is considerable variation in the amount of wind power even when aggregated at the continent level.
- The second is an analysis of how much storage would be required to produce reliable power using wind. This is also done at the aggregate level, which assumes perfect interconnection of power throughout Europe.

The goal is to study the feasibility of powering Europe with 100% wind and provide estimates for how much storage, overbuild or backup capacity would be required to do so in a way that provides reliable electricity.

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
- It assumes the required demand is a constant 1200 GW, the rough primary power use of Europe. So this analysis lets us model how much storage would be needed to meet all of Europe's energy needs.
  - This is done by taking the current amount of installed capacity in the dataset, 29.96 GW average, and multiplying the power output by 40.05 to produce 1200 GW average output.
  - This is not a perfect estimate (distribution matters), but serves as a good enough approximation for these purposes. Given there are now wind turbines all across Europe it is reasonable to estimate that if you built twice as many wind turbines you would get roughly twice as much power as you do today, with roughly the same profile of hourly output variation.
- It assumes the storage is perfectly efficient (every 1 GWh stored can be released at any time later as 1 GWh).
- It simulates the system with the storage. 
  - If wind power is generated in excess of the demand that extra is stored (up to the capacity of the storage) 
  - If insufficient wind power is available to meet the demand then energy is drawn down from the storage as needed. The amount of energy stored will never go negative but instead there can be a shortfall in matching the demand if not enough energy is stored.
- Some of these scenarios simulate overbuild: building additional extra capacity.
  - An overbuild factor of 2 means that (for example) instead of meeting 25 GW of demand by building 100 GW at 25% capacity factor you would instead build 200 GW of turbines to meet that 25 GW demand. This improves the reliability of the power generated at the cost of building extra turbines.
- For a given amount of storage the analysis calculates:
  - The minimal amount of overbuild that would ensure that supply always meets demand.  
    - This is found using the [bisection method](https://en.wikipedia.org/wiki/Bisection_method) to zero in on the correct amount of overbuild.     
  - Without any overbuild: how much backup capacity would be required to ensure supply always meets demand. 
    - This is found by noting what the largest shortfall seen is when trying to draw power from the storage.
  - With a 2x overbuild: how much backup capacity would be required
  
The results:
```
Storage (GWh)   Overbuild factor required   Backup required (GW)   Backup required 2x overbuild (GW)

0               11.04                       1091.31                982.63
100             10.34                       1091.31                982.63
200             9.89                        1091.31                982.63
500             8.74                        1091.31                982.63
1000            7.59                        1091.31                982.63
2000            6.14                        1091.31                982.63
5000            4.29                        1091.31                982.63
10000           3.26                        1091.31                982.63
20000           2.88                        1091.31                982.63
50000           2.22                        1091.31                832.58
100000          1.80                        1091.31                0.00

Storage required for no overbuild (GWh): 4761097.8
```
Meaning of the columns:
- `Storage (GWh)` - the amount of storage available in GWh
- `Overbuild factor required`- given this amount of storage, and the wind data, how much overbuild is required to ensure that the demand (constant 29.96GW) is always met without any kind of backup capacity. 
- `Backup required (GW)` - without any overbuild how much backup capacity is required to ensure the demand is always met.
- `Backup required 2x overbuild (GW)` - with a 2x overbuild how much backup capacity is required to ensure the demand is always met.

Analysis:
- With no storage you must either overbuild by a factor of 11.04, or provide 1091.31 GW of backup. If you overbuild by a factor of 2 then 982.63 GW of backup capacity would need to be available. 
  - Note that 11.04 agrees with the moving average analysis that the 1h moving average has a minimum of 9.06% of the average value (1.0/0.0906 = 11.04). 
- With 100 GWh of storage (NOTE: [all the storage batteries in the world in 2018 amounted to 8 GWh](https://www.worldenergy.org/assets/downloads/ESM_Final_Report_05-Nov-2019.pdf)) you must overbuild by a factor of 10.34 or alternatively provide 1091.31 GW of backup.
- With 500 GWh of storage (NOTE: this is [more storage than all the pumped storage in Europe](https://www.dnv.com/news/estorage-study-shows-huge-potential-capacity-of-exploitable-pumped-hydro-energy-storage-sites-in-europe-63675)) you would need to overbuild by a factor of 8.74.
- With 10,000 GWh of storage (NOTE: this is [more storage than all the pumped storage in the world](https://www.hydropower.org/factsheets/pumped-storage)) you would need to overbuild by a factor of 3.26. 
- With 4,761,097.8 GWh of storage no overbuild or backup capacity is required to provide the reliable 1200 GW.

## Conclusions 

In conclusion:
- Periods of highly depressed wind output across all of Europe are common place, there are many examples in the data set. This is true both at short term frames (1 hour) and long time frames (25 days).
- This means for wind to provide reliable output it must either a) have large amounts of storage, b) massively overbuild, or c) have significant on-demand backup capacity available.
- You can trade off somewhat between these three options: but any of them is going to be expensive.
- The cheapest option (by far) is to have backup capacity available on standby: this is what Germany has done in practice. With current available technology this backup must either by hydropower, which is limited in scale, or more commonly it is met by fossil fuels (as Germany is doing).
- This analysis has just summed the power across the countries and assumed perfect interconnection is available for free at any capacity that's required: in reality it's not and adding that interconnection would add significantly to the expense.
- VRE can be cheap as a modest part of a flexible grid, but trying to get it to produce reliable electricity is extremely hard. Current LCOE figures cannot be taken as a reasonable guide to what would be required to produce reliable electricity using intermittent sources.
