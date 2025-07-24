use aws_config::BehaviorVersion;
use aws_config::SdkConfig;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cloudformation as cloudformation;
use aws_sdk_dynamodb as dynamodb;
use std::error::Error;

// Get the AWS config to grab stack details from
async fn create_config() -> Result<SdkConfig, Box<dyn Error>> {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-2");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    return Ok(config);
}

pub async fn list_tables() -> Result<Vec<String>, dynamodb::Error> {
    println!("Running list tables function...");

    let config = create_config().await.unwrap();
    let client = dynamodb::Client::new(&config);
    let resp = client.list_tables().send().await.unwrap();

    Ok(resp.table_names().to_vec())
}

pub async fn get_stack() -> Result<(), cloudformation::Error> {
    let config = create_config().await.unwrap();
    let client = cloudformation::Client::new(&config);

    let resp = client.list_stacks().send().await.unwrap();

    let stacks = resp.stack_summaries.unwrap();

    let resp = client
        .describe_stacks()
        .stack_name(stacks[2].stack_name().unwrap())
        .send()
        .await?;

    let stack = resp.stacks().first().unwrap();
    println!("{:#?}", stack);

    Ok(())
}
