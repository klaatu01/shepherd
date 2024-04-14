use aws_sdk_eventbridge::types::RuleState;

use crate::core::{EventSourceMapping, EventSourceMappingState, Lambda};

pub async fn list_eventbuses(client: &aws_sdk_eventbridge::Client) -> Vec<String> {
    let mut buses = Vec::new();
    let mut next_token = None;
    loop {
        let response = client
            .list_event_buses()
            .set_next_token(next_token)
            .send()
            .await
            .unwrap();
        buses.extend(response.event_buses().iter().cloned());
        next_token = response.next_token().map(|s| s.to_string());
        if next_token.is_none() {
            break;
        }
    }
    buses
        .into_iter()
        .map(|bus| bus.name().unwrap().to_string())
        .collect()
}

pub async fn event_source_mappings(
    client: &aws_sdk_eventbridge::Client,
    lambda: &Lambda,
) -> Vec<EventSourceMapping> {
    let event_buses = list_eventbuses(client).await;

    let rule_futures = event_buses.iter().map(|bus| async {
        let mut rules = Vec::new();
        let mut next_token = None;
        let bus = bus.clone();
        loop {
            let response = client
                .list_rules()
                .event_bus_name(bus.clone())
                .set_next_token(next_token.clone())
                .send()
                .await
                .unwrap();
            rules.extend(response.rules().iter().cloned());
            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }
        rules
    });

    let rules = futures::future::join_all(rule_futures).await;
    let rules = rules.into_iter().flatten().collect::<Vec<_>>();

    let targets = rules.iter().map(|rule| async {
        let rule_name = rule.name().clone().unwrap();
        let event_bus_name = rule.event_bus_name().clone().map(|s| s.to_string());
        let targets = client
            .list_targets_by_rule()
            .rule(rule_name)
            .set_event_bus_name(event_bus_name)
            .send()
            .await;
        (rule.clone(), targets)
    });

    let results = futures::future::join_all(targets).await;
    let results = results
        .into_iter()
        .filter(|(_, targets)| targets.is_ok())
        .map(|(rule, targets)| (rule, targets.unwrap()))
        .filter(|(rule, targets)| {
            let targets = targets.targets();
            targets.iter().any(|target| {
                let lambda_arn = &lambda.arn;
                let raw_lambda = target.arn().replace(":Live", "");
                raw_lambda == lambda_arn.as_str()
            })
        })
        .collect::<Vec<_>>();

    results
        .iter()
        .map(|(rule, target)| {
            let name = rule.name().clone().unwrap().to_string();
            let event_bus_name = rule.event_bus_name().clone().map(|s| s.to_string());

            let state = match rule.state().unwrap() {
                RuleState::Enabled => EventSourceMappingState::Enabled,
                RuleState::Disabled => EventSourceMappingState::Disabled,
                _ => EventSourceMappingState::Disabled,
            };

            EventSourceMapping::EventBridge {
                name,
                event_bus_name: event_bus_name.unwrap_or("default".to_string()),
                state,
            }
        })
        .collect()
}
