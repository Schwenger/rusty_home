// use crate::api::HomeEditError;

#[derive(Debug)]
pub enum HomeBaseError {
  Io(std::io::Error),
  Serde(serde_yaml::Error),
  Paho(paho_mqtt::Error),
  CtrlC(ctrlc::Error),
  ImpossibleStrConversion,
  InvalidTopic,
  // HomeEdit(crate:::api::HomeEditError),
}

impl From<std::io::Error> for HomeBaseError {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl From<serde_yaml::Error> for HomeBaseError {
  fn from(value: serde_yaml::Error) -> Self {
    Self::Serde(value)
  }
}

impl From<paho_mqtt::Error> for HomeBaseError {
  fn from(value: paho_mqtt::Error) -> Self {
    Self::Paho(value)
  }
}

impl From<ctrlc::Error> for HomeBaseError {
  fn from(value: ctrlc::Error) -> Self {
    Self::CtrlC(value)
  }
}
