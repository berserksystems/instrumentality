![Instrumentality](./assets/dark-header.png#gh-dark-mode-only)
![](./assets/light-header.png#gh-light-mode-only)
---
Instrumentality facilitates the aggregation of data from any source into a 
single database under a common set of schemas.

## Thesis.
Data should belong to people and those they choose to share it with. The order
in which posts are presented should be changed from reverse chronological order
 (latest first) only when the user expressly wishes to do so.

## Documentation.
For server administrators: <https://docs.berserksystems.com/>.

For developers: <https://docs.rs/instrumentality/>.

## Download.
See <https://github.com/berserksystems/instrumentality/releases/>.

## Architecture.
This is an Axum web server that reads and writes data to MongoDB. Users register
with the platform and create subjects. Subjects can be organised into groups.
Data is about subjects and is continuous, discrete or meta.

## Features.
- Abstraction over common data: content, presence, metadata.
- Abstraction over people and organisations: group and subjects.
- Full TLS support.
- Basic authentication through API keys.
- Registration through referral.
- Basic data verification.
- Queue system for prioritising jobs.

### Roadmap.
#### Ecosystem.
- [ ] Provider clients: Python, Rust.
- [ ] Consumer clients: Web frontend.

#### Features.
- [ ] Performance profiling and load testing.
- [ ] Migrate to PostgreSQL. 
- [ ] Enhanced `/view` query syntax.
- [ ] Hot and cold `/queue`.
- [ ] `/leaderboard`.
- [ ] Composable groups.
- [ ] Webhooks.
- [ ] Analytics.
- [ ] Admin tooling.
- [ ] Configuration file updating workflow.
- [ ] Byzantine consensus.