#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Scalar(pub(self) f64);
impl From<f64> for Scalar {
  fn from(value: f64) -> Self {
    assert!((0.0..=1.0).contains(&value));
    Self(value)
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
}
