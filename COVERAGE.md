This file serves to document our current test coverage status. The vast majority
of our tests are integration tests.

The following is the output of `cargo llvm-cov`, which can be installing by 
running `cargo install cargo-llvm-cov`.

```
Filename                       Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
boot.rs                             24                24     0.00%           5                 5     0.00%          25                25     0.00%           0                 0         -
config.rs                           70                18    74.29%          22                 5    77.27%          78                27    65.38%           0                 0         -
data.rs                             98                12    87.76%          15                 3    80.00%         167                11    93.41%           0                 0         -
database.rs                         91                16    82.42%          26                 4    84.62%         187                 8    95.72%           0                 0         -
group.rs                            16                 4    75.00%           4                 2    50.00%           4                 2    50.00%           0                 0         -
lib.rs                               1                 0   100.00%           1                 0   100.00%           0                 0         -           0                 0         -
main.rs                              4                 3    25.00%           3                 2    33.33%           3                 3     0.00%           0                 0         -
routes/add.rs                       53                 3    94.34%           6                 0   100.00%          92                 3    96.74%           0                 0         -
routes/default.rs                   11                 1    90.91%           4                 0   100.00%          18                 4    77.78%           0                 0         -
routes/frontpage.rs                  2                 0   100.00%           2                 0   100.00%           4                 0   100.00%           0                 0         -
routes/groups/create.rs             37                 6    83.78%           8                 2    75.00%          44                 3    93.18%           0                 0         -
routes/groups/delete.rs             23                 7    69.57%           6                 2    66.67%          35                 6    82.86%           0                 0         -
routes/groups/update.rs             33                 5    84.85%           6                 2    66.67%          58                10    82.76%           0                 0         -
routes/halt.rs                       9                 0   100.00%           4                 0   100.00%          22                 0   100.00%           0                 0         -
routes/queue.rs                    127                19    85.04%          24                 3    87.50%         281                26    90.75%           0                 0         -
routes/response.rs                 105                19    81.90%          31                 1    96.77%          99                 1    98.99%           0                 0         -
routes/subjects/create.rs           39                 8    79.49%           8                 2    75.00%          45                 3    93.33%           0                 0         -
routes/subjects/delete.rs           36                 8    77.78%           6                 2    66.67%          50                 8    84.00%           0                 0         -
routes/subjects/update.rs           49                 4    91.84%           8                 2    75.00%          69                 6    91.30%           0                 0         -
routes/types.rs                      2                 0   100.00%           2                 0   100.00%           7                 0   100.00%           0                 0         -
routes/user/login.rs                11                 0   100.00%           2                 0   100.00%           8                 0   100.00%           0                 0         -
routes/user/reset.rs                11                 1    90.91%           2                 0   100.00%          21                 4    80.95%           0                 0         -
routes/users/invite.rs              31                 4    87.10%          11                 2    81.82%          49                 2    95.92%           0                 0         -
routes/users/register.rs            55                 5    90.91%          12                 2    83.33%          70                 4    94.29%           0                 0         -
routes/view.rs                      92                16    82.61%          15                 0   100.00%         126                 3    97.62%           0                 0         -
server.rs                           47                20    57.45%          15                 6    60.00%         112                20    82.14%           0                 0         -
subject.rs                          16                 3    81.25%           4                 1    75.00%           4                 1    75.00%           0                 0         -
user.rs                             78                 7    91.03%          20                 1    95.00%         114                 2    98.25%           0                 0         -
utils/deserialise_array.rs          12                 1    91.67%           4                 0   100.00%          22                 0   100.00%           0                 0         -
utils/random.rs                      9                 0   100.00%           5                 0   100.00%          29                 0   100.00%           0                 0         -
------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                             1192               214    82.05%         281                49    82.56%        1843               182    90.12%           0                 0         -
```
