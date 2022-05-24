use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub formats: BTreeMap<String, String>,
    pub db: BTreeMap<String, String>,
}
