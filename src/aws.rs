use aws_config::BehaviorVersion;
use aws_config::SdkConfig;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation as cloudformation;
use aws_sdk_cloudformation::types::{StackResource, StackSummary};
use aws_sdk_dynamodb as dynamodb;
use std::error::Error;
use std::sync::Arc;

/// Get the AWS config to grab stack details from default default_provider
// TODO: We need to give the user the ability to run this and pass in a specific profile similar to
// serverless. Or better yet, we read all the profiles in the local machine and have the ability to
// navigate everything.
async fn create_config() -> Result<SdkConfig, Box<dyn Error>> {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-2");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    return Ok(config);
}

pub async fn _list_tables() -> Result<Vec<String>, dynamodb::Error> {
    println!("Running list tables function...");

    let config = create_config().await.unwrap();
    let client = dynamodb::Client::new(&config);
    let resp = client.list_tables().send().await.unwrap();

    Ok(resp.table_names().to_vec())
}

/// Gets the stack summaries for all stacks on the given profile
pub async fn get_stacks() -> Result<Vec<StackSummary>, cloudformation::Error> {
    let config = create_config().await.unwrap();
    let client = cloudformation::Client::new(&config);

    let resp = client.list_stacks().send().await.unwrap();

    let stacks = resp.stack_summaries.unwrap();
    Ok(stacks)
}

pub async fn get_stack_resources(
    stack: Arc<StackSummary>,
) -> Result<Vec<StackResource>, cloudformation::Error> {
    let config = create_config().await.unwrap();
    let client = cloudformation::Client::new(&config);

    let resp = client
        .describe_stack_resources()
        .stack_name(stack.stack_name().unwrap())
        .send()
        .await
        .unwrap();
    let stack_resources = resp.stack_resources.unwrap();
    Ok(stack_resources)
}
