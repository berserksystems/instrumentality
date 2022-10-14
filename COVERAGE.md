This file serves to document our current test coverage status. The majority of
our tests are integration tests.

The following is the output of `cargo llvm-cov`, which can be installing by 
running `cargo install cargo-llvm-cov`.

```
Filename                       Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
config.rs                           68                15    77.94%          21                 3    85.71%          59                13    77.97%           0                 0         -    
data.rs                             98                13    86.73%          15                 3    80.00%         167                11    93.41%           0                 0         -    
database.rs                         89                14    84.27%          25                 4    84.00%         219                 9    95.89%           0                 0         -    
group.rs                            14                 3    78.57%           4                 2    50.00%           4                 2    50.00%           0                 0         -    
lib.rs                               1                 0   100.00%           1                 0   100.00%           1                 0   100.00%           0                 0         -    
main.rs                             29                28     3.45%           8                 7    12.50%          30                29     3.33%           0                 0         -    
response.rs                        112                31    72.32%          31                 1    96.77%         100                 3    97.00%           0                 0         -    
routes\add.rs                       49                 4    91.84%           6                 0   100.00%          85                 4    95.29%           0                 0         -    
routes\create.rs                    67                13    80.60%          10                 2    80.00%         105                22    79.05%           0                 0         -
routes\default.rs                   11                 1    90.91%           4                 0   100.00%          25                 7    72.00%           0                 0         -    
routes\delete.rs                    39                 5    87.18%           6                 2    66.67%          60                13    78.33%           0                 0         -    
routes\frontpage.rs                  2                 0   100.00%           2                 0   100.00%           4                 0   100.00%           0                 0         -
routes\halt.rs                       9                 0   100.00%           4                 0   100.00%          21                 0   100.00%           0                 0         -
routes\invite.rs                    32                 3    90.62%          12                 2    83.33%          49                 2    95.92%           0                 0         -
routes\login.rs                      9                 0   100.00%           2                 0   100.00%           7                 0   100.00%           0                 0         -
routes\queue.rs                    124                15    87.90%          24                 3    87.50%         274                31    88.69%           0                 0         -
routes\register.rs                  55                 8    85.45%          12                 2    83.33%          64                 7    89.06%           0                 0         -
routes\reset.rs                      9                 1    88.89%           2                 0   100.00%          21                 7    66.67%           0                 0         -
routes\types.rs                      2                 0   100.00%           2                 0   100.00%           6                 0   100.00%           0                 0         -
routes\update.rs                    85                 9    89.41%          12                 2    83.33%         126                16    87.30%           0                 0         -
routes\view.rs                      97                16    83.51%          18                 1    94.44%         117                 1    99.15%           0                 0         -
server.rs                           46                20    56.52%          16                 7    56.25%          92                24    73.91%           0                 0         -
subject.rs                          14                 2    85.71%           4                 1    75.00%           4                 1    75.00%           0                 0         -
user.rs                             78                 4    94.87%          21                 1    95.24%         101                 1    99.01%           0                 0         -
utils\deserialise_array.rs          12                 1    91.67%           4                 0   100.00%          22                 0   100.00%           0                 0         -
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ 
TOTAL                             1151               206    82.10%         266                43    83.83%        1763               203    88.49%           0                 0         -
```