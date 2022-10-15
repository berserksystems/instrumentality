This file serves to document our current test coverage status. The majority of
our tests are integration tests.

The following is the output of `cargo llvm-cov`, which can be installing by 
running `cargo install cargo-llvm-cov`.

```
Filename                       Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ 
boot.rs                             24                24     0.00%           5                 5     0.00%          25                25     0.00%           0                 0         -
config.rs                           68                15    77.94%          21                 3    85.71%          59                13    77.97%           0                 0         -
data.rs                             98                13    86.73%          15                 3    80.00%         167                11    93.41%           0                 0         -
database.rs                         83                14    83.13%          23                 4    82.61%         196                 9    95.41%           0                 0         -
group.rs                            14                 3    78.57%           4                 2    50.00%           4                 2    50.00%           0                 0         -
lib.rs                               1                 0   100.00%           1                 0   100.00%           1                 0   100.00%           0                 0         -
main.rs                              5                 4    20.00%           3                 2    33.33%           5                 4    20.00%           0                 0         -
routes\add.rs                       49                 3    93.88%           6                 0   100.00%          91                 3    96.70%           0                 0         -    
routes\create.rs                    67                13    80.60%          10                 2    80.00%         105                22    79.05%           0                 0         -    
routes\default.rs                   11                 1    90.91%           4                 0   100.00%          25                 7    72.00%           0                 0         -    
routes\delete.rs                    39                 5    87.18%           6                 2    66.67%          60                13    78.33%           0                 0         -    
routes\frontpage.rs                  2                 0   100.00%           2                 0   100.00%           4                 0   100.00%           0                 0         -    
routes\halt.rs                       9                 0   100.00%           4                 0   100.00%          21                 0   100.00%           0                 0         -    
routes\invite.rs                    32                 3    90.62%          12                 2    83.33%          49                 2    95.92%           0                 0         -    
routes\login.rs                      9                 0   100.00%           2                 0   100.00%           7                 0   100.00%           0                 0         -    
routes\queue.rs                    120                15    87.50%          24                 3    87.50%         275                31    88.73%           0                 0         -    
routes\register.rs                  55                 8    85.45%          12                 2    83.33%          64                 7    89.06%           0                 0         -    
routes\reset.rs                      9                 1    88.89%           2                 0   100.00%          21                 7    66.67%           0                 0         -    
routes\types.rs                      2                 0   100.00%           2                 0   100.00%           6                 0   100.00%           0                 0         -    
routes\update.rs                    85                 9    89.41%          12                 2    83.33%         126                16    87.30%           0                 0         -    
routes\view.rs                      97                16    83.51%          18                 1    94.44%         117                 1    99.15%           0                 0         -    
server.rs                           46                20    56.52%          16                 7    56.25%          92                24    73.91%           0                 0         -    
subject.rs                          14                 2    85.71%           4                 1    75.00%           4                 1    75.00%           0                 0         -    
user.rs                             78                 4    94.87%          21                 1    95.24%         101                 1    99.01%           0                 0         -    
utils\deserialise_array.rs          12                 1    91.67%           4                 0   100.00%          22                 0   100.00%           0                 0         -    
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------                                                                                                                                                        --------------------------------------
TOTAL                             1141               205    82.03%         264                43    83.71%        1747               202    88.44%           0                 0         -    
```