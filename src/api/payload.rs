use serde::Serialize;

#[derive(Debug, Clone)]
pub struct JsonPayload(String);

impl JsonPayload {
  pub fn from<T: Serialize>(v: &T) -> Self {
    Self(serde_json::to_string(v).expect("Why would this fail?"))
  }
  pub fn inner(&self) -> &str {
    &self.0
  }
  pub fn from_string(inner: String) -> Self {
    Self(inner)
  }
  pub fn to_str(&self) -> String {
    String::from(self.inner())
  }
}
