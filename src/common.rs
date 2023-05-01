use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Scalar(pub(self) f64);
impl From<f64> for Scalar {
  fn from(value: f64) -> Self {
    assert!((-0.01..=1.1).contains(&value), "Value {value} is not a scalar.");
    Self::bounded(value)
  }
}

impl From<Scalar> for f64 {
  fn from(value: Scalar) -> Self {
    value.0
  }
}

impl Scalar {
  pub fn inner(&self) -> f64 {
    self.0
  }

  fn bounded(value: f64) -> Self {
    Self(value.min(1f64).max(0f64))
  }
}

impl Add<f64> for Scalar {
  type Output = Self;

  fn add(self, rhs: f64) -> Self::Output {
    Self::bounded(self.inner() + rhs)
  }
}

impl Sub<f64> for Scalar {
  type Output = Self;

  fn sub(self, rhs: f64) -> Self::Output {
    Self::bounded(self.inner() - rhs)
  }
}

impl AddAssign<f64> for Scalar {
  fn add_assign(&mut self, rhs: f64) {
    let val = *self + rhs;
    self.0 = val.inner()
  }
}

impl SubAssign<f64> for Scalar {
  fn sub_assign(&mut self, rhs: f64) {
    let val = *self - rhs;
    self.0 = val.inner()
  }
}

impl Mul for Scalar {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self::Output {
    Self(self.inner() * rhs.inner())
  }
}

impl MulAssign for Scalar {
  fn mul_assign(&mut self, rhs: Self) {
    self.0 = self.inner() * rhs.inner()
  }
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub enum Tertiary<T: std::fmt::Debug + Clone + Serialize + PartialEq> {
  Some(T),
  #[default]
  None,
  Query,
}

impl<T: std::fmt::Debug + Clone + Serialize + PartialEq> Tertiary<T> {
  pub fn map<R, F>(self, f: F) -> Tertiary<R>
  where
    R: std::fmt::Debug + Clone + Serialize + PartialEq,
    F: FnOnce(T) -> R,
  {
    match self {
      Tertiary::None => Tertiary::None,
      Tertiary::Query => Tertiary::Query,
      Tertiary::Some(x) => Tertiary::Some(f(x)),
    }
  }
  pub fn to_json(&self) -> Option<Value> {
    match self {
      Tertiary::None => None,
      Tertiary::Query => Some(json!("")),
      Tertiary::Some(v) => Some(serde_json::to_value(v).unwrap()),
    }
  }
}
