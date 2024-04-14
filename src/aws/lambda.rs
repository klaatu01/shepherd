use anyhow::Result;

use crate::core::{EventSourceMapping, EventSourceMappingState, Lambda};
use std::{fs, io::Write, path::Path};

fn read_lambdas_from_cache() -> Option<Vec<Lambda>> {
    let path = format!(
        "{}/.config/shepherd/lambdas.json",
        dirs::home_dir().unwrap().to_str().unwrap()
    );

    if !Path::new(&path).exists() {
        return None;
    }

    let content = fs::read_to_string(&path).unwrap();
    let lambda_functions: Vec<Lambda> = serde_json::from_str(&content).unwrap();

    Some(lambda_functions)
}

fn write_lambdas_to_cache(lambda_functions: &Vec<Lambda>) {
    let path = format!(
        "{}/.config/shepherd/lambdas.json",
        dirs::home_dir().unwrap().to_str().unwrap()
    );

    if let Some(cache) = Path::new(&path).parent() {
        fs::create_dir_all(cache).unwrap();
    }

    let content = serde_json::to_string(lambda_functions).unwrap();

    let mut file = fs::File::create(&path).unwrap();

    file.write_all(content.as_bytes()).unwrap();
}

async fn fetch_lambdas(client: &aws_sdk_lambda::Client) -> Result<Vec<Lambda>> {
    let mut lambda_functions: Vec<Lambda> = Vec::new();
    let mut next_marker = None;

    loop {
        let response = client
            .list_functions()
            .max_items(50)
            .set_marker(next_marker)
            .send()
            .await?;

        let functions = response.functions().iter().map(|f| Lambda {
            timeout: f.timeout.unwrap() as i64,
            runtime: f.runtime.clone().unwrap().to_string(),
            memory: f.memory_size.unwrap() as i64,
            name: f.function_name.clone().unwrap(),
            arn: f.function_arn.clone().unwrap(),
        });

        lambda_functions.extend(functions);

        next_marker = response.next_marker.clone();
        if next_marker.is_none() {
            break;
        }
    }

    Ok(lambda_functions)
}

pub(crate) fn clear_cache() {
    let path = format!(
        "{}/.config/shepherd/lambdas.json",
        dirs::home_dir().unwrap().to_str().unwrap()
    );

    if Path::new(&path).exists() {
        fs::remove_file(&path).unwrap();
    }
}

pub(crate) async fn lambda_functions(client: &aws_sdk_lambda::Client) -> Result<Vec<Lambda>> {
    let cache = read_lambdas_from_cache();

    if let Some(lambda_functions) = cache {
        return Ok(lambda_functions);
    }

    let lambda_functions = fetch_lambdas(client).await?;
    write_lambdas_to_cache(&lambda_functions);

    Ok(lambda_functions)
}

pub(crate) async fn lambda_event_source_mappings(
    client: &aws_sdk_lambda::Client,
    lambda_name: &str,
) -> Result<Vec<EventSourceMapping>> {
    let response = client
        .list_event_source_mappings()
        .function_name(lambda_name)
        .send()
        .await?;

    let mappings = response.event_source_mappings().iter().flat_map(|m| {
        println!("{:?}", m);
        let event_source = m
            .event_source_arn()
            .unwrap()
            .split(':')
            .nth(2)
            .unwrap()
            .to_string();
        let name = m
            .event_source_arn()
            .unwrap()
            .split(':')
            .last()
            .unwrap()
            .to_string();

        match event_source.as_ref() {
            "sqs" => Some(EventSourceMapping::SQS {
                name,
                batch_size: m.batch_size.unwrap() as i64,
                batch_window: m.maximum_batching_window_in_seconds().unwrap() as i64,
                state: match m.state().unwrap().as_ref() {
                    "Disabled" => EventSourceMappingState::Disabled,
                    _ => EventSourceMappingState::Enabled,
                },
            }),
            "events" => {
                let split = name.split('/').collect::<Vec<&str>>();

                Some(EventSourceMapping::EventBridge {
                    name: split.last().unwrap().to_string(),
                    event_bus_name: split.first().unwrap().to_string(),
                    state: match m.state().unwrap().as_ref() {
                        "Disabled" => EventSourceMappingState::Disabled,
                        _ => EventSourceMappingState::Enabled,
                    },
                })
            }
            _ => None,
        }
    });

    Ok(mappings.collect())
}
