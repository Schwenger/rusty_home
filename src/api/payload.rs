pub struct JsonPayload(String);

impl JsonPayload {
  pub fn from<T: serde::Serialize>(v: &T) -> Self {
    Self(serde_json::to_string(v).expect("Why would this fail?"))
  }
}

#[allow(dead_code)]
pub struct MqttPayload;
