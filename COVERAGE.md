This file serves to document our current test coverage status. The vast majority
of our tests are integration tests.

The following is the output of `cargo llvm-cov`, which can be installing by 
running `cargo install cargo-llvm-cov`.

```
Filename                       Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
boot.rs                             24                24     0.00%           5                 5     0.00%          25                25     0.00%           0                 0         -
config.rs                           76                27    64.47%          22                 5    77.27%          78                27    65.38%           0                 0         -
data.rs                             98                13    86.73%          15                 3    80.00%         167                11    93.41%           0                 0         -
database.rs                         91                16    82.42%          26                 4    84.62%         182                 8    95.60%           0                 0         -
group.rs                            14                 3    78.57%           4                 2    50.00%           4                 2    50.00%           0                 0         -
lib.rs                               1                 0   100.00%           1                 0   100.00%           1                 0   100.00%           0                 0         -
main.rs                              4                 3    25.00%           3                 2    33.33%           4                 3    25.00%           0                 0         -
routes/add.rs                       53                 3    94.34%           6                 0   100.00%          92                 3    96.74%           0                 0         -
routes/create.rs                    70                13    81.43%          10                 2    80.00%          99                11    88.89%           0                 0         -
routes/default.rs                   11                 1    90.91%           4                 0   100.00%          18                 4    77.78%           0                 0         -
routes/delete.rs                    43                 5    88.37%           6                 2    66.67%          68                 8    88.24%           0                 0         -
routes/frontpage.rs                  2                 0   100.00%           2                 0   100.00%           4                 0   100.00%           0                 0         -
routes/halt.rs                       9                 0   100.00%           4                 0   100.00%          22                 0   100.00%           0                 0         -
routes/invite.rs                    29                 3    89.66%          11                 2    81.82%          49                 2    95.92%           0                 0         -
routes/login.rs                     11                 0   100.00%           2                 0   100.00%           8                 0   100.00%           0                 0         -
routes/queue.rs                    122                15    87.70%          24                 3    87.50%         281                26    90.75%           0                 0         -
routes/register.rs                  57                 8    85.96%          12                 2    83.33%          70                 4    94.29%           0                 0         -
routes/reset.rs                     11                 1    90.91%           2                 0   100.00%          21                 4    80.95%           0                 0         -
routes/response.rs                 113                29    74.34%          31                 1    96.77%          99                 1    98.99%           0                 0         -
routes/types.rs                      2                 0   100.00%           2                 0   100.00%           7                 0   100.00%           0                 0         -
routes/update.rs                    89                 9    89.89%          12                 2    83.33%         134                12    91.04%           0                 0         -
routes/view.rs                      93                15    83.87%          15                 0   100.00%         126                 3    97.62%           0                 0         -
server.rs                           47                20    57.45%          15                 6    60.00%          88                20    77.27%           0                 0         -
subject.rs                          14                 2    85.71%           4                 1    75.00%           4                 1    75.00%           0                 0         -
user.rs                             76                 6    92.11%          20                 1    95.00%         110                 2    98.18%           0                 0         -
utils/deserialise_array.rs          12                 1    91.67%           4                 0   100.00%          22                 0   100.00%           0                 0         -
utils/random.rs                      9                 0   100.00%           5                 0   100.00%          29                 0   100.00%           0                 0         -
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                             1181               217    81.63%         267                43    83.90%        1812               177    90.23%           0                 0         -
```