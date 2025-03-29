use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use prometheus_client::encoding::{EncodeLabelValue, LabelValueEncoder};

#[derive(Eq, Hash, PartialEq, Clone)]
pub(crate) struct ContainerId(String);

impl From<String> for ContainerId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ContainerId {
    pub(crate) fn get_short(&self) -> &str {
        &self.0[..12]
    }
}

impl Deref for ContainerId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for ContainerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_short())
    }
}

impl EncodeLabelValue for ContainerId {
    fn encode(&self, encoder: &mut LabelValueEncoder) -> Result<(), Error> {
        EncodeLabelValue::encode(&self.0.as_str(), encoder)
    }
}
