use std::fmt::{Debug, Formatter};

#[derive(Eq, Hash, PartialEq, Clone)]
pub(crate) struct ContainerId(String);

impl From<String> for ContainerId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ContainerId {
    pub(crate) fn get(&self) -> &str {
        self.0.as_str()
    }

    pub(crate) fn get_short(&self) -> &str {
        &self.0[..12]
    }
}

impl Debug for ContainerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_short())
    }
}
