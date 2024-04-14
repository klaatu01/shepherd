use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use aws_sdk_cloudwatch::types::{
    builders::{MetricBuilder, MetricDataQueryBuilder, MetricStatBuilder},
    Dimension, MetricDataQuery,
};

use crate::core::Metric;

pub fn build_metric(
    metric_id: &str,
    lambda_name: &str,
    metric_name: &str,
    stat: &str,
    period: i32,
) -> MetricDataQuery {
    MetricDataQueryBuilder::default()
        .id(metric_id)
        .metric_stat(
            MetricStatBuilder::default()
                .stat(stat)
                .metric(
                    MetricBuilder::default()
                        .namespace("AWS/Lambda")
                        .metric_name(metric_name)
                        .dimensions(
                            Dimension::builder()
                                .name("FunctionName")
                                .value(lambda_name)
                                .build(),
                        )
                        .build(),
                )
                .period(period)
                .build(),
        )
        .build()
}

// get invocations of a lambda for the past 24 hours
pub async fn metrics(client: &aws_sdk_cloudwatch::Client, arn: &String) -> Result<Vec<Metric>> {
    let period = 60;

    let start_time = SystemTime::now()
        .checked_sub(Duration::from_secs(86400))
        .unwrap();
    let end_time = SystemTime::now();

    let response = client
        .get_metric_data()
        .metric_data_queries(build_metric(
            "invocations",
            arn,
            "Invocations",
            "Sum",
            period,
        ))
        .metric_data_queries(build_metric("errors", arn, "Errors", "Sum", period))
        .metric_data_queries(build_metric("duration", arn, "Duration", "Average", period))
        .metric_data_queries(build_metric(
            "concurrent_executions",
            arn,
            "ConcurrentExecutions",
            "Maximum",
            period,
        ))
        .start_time(start_time.into())
        .end_time(end_time.into())
        .send()
        .await?;

    let start_timestamp = start_time.duration_since(UNIX_EPOCH)?.as_secs();
    let end_timestamp = end_time.duration_since(UNIX_EPOCH)?.as_secs();

    let all_timestamps: Vec<u64> = (start_timestamp..=end_timestamp)
        .step_by(period as usize)
        .map(|x| x as u64 / period as u64)
        .collect();

    let respone: Vec<Result<_>> = response
        .metric_data_results()
        .iter()
        .map(|metric| {
            let mut hashmap: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();

            for timestamp in all_timestamps.iter() {
                hashmap.insert(*timestamp, 0.0);
            }

            for (i, timestamp) in metric.timestamps().iter().enumerate() {
                let timestamp = timestamp.secs() / period as i64;
                hashmap.insert(timestamp.try_into().unwrap(), metric.values()[i]);
            }

            let mut metrics = hashmap.iter().collect::<Vec<_>>();
            metrics.sort_by(|a, b| a.0.cmp(b.0));

            Ok(Metric {
                name: metric.id().unwrap().to_string(),
                values: metrics.iter().map(|(_, v)| **v).collect(),
                timestamps: metrics
                    .iter()
                    .map(|(k, _)| (**k * period as u64) as u64)
                    .collect(),
                metric: metric.label().unwrap().to_string(),
            })
        })
        .collect();

    Ok(respone.into_iter().collect::<Result<Vec<Metric>>>()?)
}
