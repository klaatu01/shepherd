use aws_config::BehaviorVersion;

use crate::core::{EventSourceMapping, Lambda, Metric};
use anyhow::Result;

pub(crate) mod cloudwatch;
pub(crate) mod event_bridge;
pub(crate) mod lambda;

pub struct AWS {
    pub sdk_config: aws_config::SdkConfig,
    pub lambda_client: aws_sdk_lambda::Client,
    pub cw_client: aws_sdk_cloudwatch::Client,
    pub eb_client: aws_sdk_eventbridge::Client,
}

impl AWS {
    pub async fn new() -> Self {
        let sdk_config = aws_config::defaults(BehaviorVersion::v2023_11_09())
            .load()
            .await;

        let cw_client = aws_sdk_cloudwatch::Client::new(&sdk_config);
        let lambda_client = aws_sdk_lambda::Client::new(&sdk_config);
        let eb_client = aws_sdk_eventbridge::Client::new(&sdk_config);

        Self {
            sdk_config,
            lambda_client,
            cw_client,
            eb_client,
        }
    }

    pub async fn lambda_functions(&self) -> Result<Vec<Lambda>> {
        lambda::lambda_functions(&self.lambda_client).await
    }

    pub async fn metrics(&self, lambda: &Lambda) -> Result<Vec<Metric>> {
        cloudwatch::metrics(&self.cw_client, &lambda.name).await
    }

    pub async fn event_source_mappings(&self, lambda: &Lambda) -> Result<Vec<EventSourceMapping>> {
        let eb_event_source_mappings =
            event_bridge::event_source_mappings(&self.eb_client, &lambda).await?;
        let mut event_sources =
            lambda::lambda_event_source_mappings(&self.lambda_client, &lambda.name).await?;
        event_sources.extend(eb_event_source_mappings);

        Ok(event_sources)
    }

    pub async fn clear_cache(&self) {
        lambda::clear_cache();
    }
}
