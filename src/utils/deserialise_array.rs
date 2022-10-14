// https://github.com/tokio-rs/axum/issues/434#issuecomment-954924025
// No support for vec in query. Using workaround by jplatte.

use serde::{Deserialize, Deserializer};

pub fn deserialise_array<'de, D>(
    deserializer: D,
) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let nb = s
        .chars()
        .filter(|c| !vec!['[', ']'].contains(c))
        .collect::<String>();
    let v = nb
        .split(',')
        .map(|s| s.trim())
        .map(|s| s.into())
        .collect::<Vec<String>>();

    Ok(v)
}
