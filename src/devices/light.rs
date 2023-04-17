use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightGroup {
  atomics: Vec<Light>,
  subs: Vec<LightGroup>,
}
