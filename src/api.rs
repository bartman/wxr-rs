use reqwest::Client;

use crate::models::{GraphQLRequest, GraphQLResponse, WorkoutRequest, WorkoutResponse};

pub async fn login_request(client: &Client, request: &GraphQLRequest) -> Result<GraphQLResponse, Box<dyn std::error::Error>> {
    let response = client
        .post("https://weightxreps.net/api/graphql")
        .json(request)
        .send()
        .await?;
    let body: GraphQLResponse = response.json().await?;
    Ok(body)
}

pub async fn workout_request(client: &Client, token: &str, request: &WorkoutRequest) -> Result<WorkoutResponse, Box<dyn std::error::Error>> {
    let response = client
        .post("https://weightxreps.net/api/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .json(request)
        .send()
        .await?;
    let body: WorkoutResponse = response.json().await?;
    Ok(body)
}