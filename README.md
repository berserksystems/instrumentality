![Instrumentality](./assets/dark-header.png#gh-dark-mode-only)
![](./assets/light-header.png#gh-light-mode-only)
---
Instrumentality facilitates the aggregation of data from any source into a 
single database under a common set of schemas.

## Thesis
Data should belong to people and those they choose to share it with. The order
in which posts are presented should be changed from reverse chronological order
 (latest first) only when the user expressly wishes to do so.

## Documentation
For server administrators: <https://docs.berserksystems.com/>.

For developers: <https://docs.rs/instrumentality/>.

## Download
See <https://github.com/berserksystems/instrumentality/releases/>.

## Architecture
This is an Axum web server that reads and writes data to MongoDB.

## Features
- Abstraction over common data: content, presence, metadata.
- Abstraction over people and organisations: group and subjects.
- Full TLS support.
- Basic authentication through API keys.
- Registration through referral.
- Basic data verification.
- Queue system for prioritising jobs.

### Future
#### Minor
- [ ] Hot and cold `/queue`
- [ ] Live config reloading.
- [ ] `/leaderboard`.
- [ ] Basic analytics & dashboard on `/`.
- [ ] Channels & webhooks.

#### Major
- [ ] Sharded database.
- [ ] Admin tooling.
- [ ] Example front end.
- [ ] CDN caching media.
- [ ] GraphQL for `/view`.
- [ ] Handling discrepencies/byzantine platforms through consensus.