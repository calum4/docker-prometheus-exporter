use std::fmt::{Debug, Formatter};
use std::time::Duration;
use async_trait::async_trait;
use metrics::{Counter, counter, describe_counter};
use tracing::instrument;
use crate::metrics::Metric;

pub(crate) struct Test {
    metric: Counter,
}

impl Test {
    pub(crate) fn new() -> Self {
        let gauge = counter!(Test::NAME);
        describe_counter!(Test::NAME, Test::DESCRIPTION);

        Test {
            metric: gauge,
        }
    }
}

#[async_trait]
impl Metric for Test {
    const NAME: &'static str = "test";
    const DESCRIPTION: &'static str = "This is a test!";
    const INTERVAL: Duration = Duration::from_secs(5);

    #[instrument]
    async fn update(&self) {
        self.metric.increment(1);
    }
}

impl Debug for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metric")
            .field("name", &Self::NAME)
            .finish()
    }
}
