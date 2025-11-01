use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json;

use crate::models::{GraphQLRequest, GraphQLResponse, WorkoutRequest, WorkoutResponse};

#[async_trait]
pub trait ApiClient: Send + Sync {
    async fn login_request(&self, request: &GraphQLRequest) -> Result<GraphQLResponse<crate::models::LoginData>, Box<dyn std::error::Error>>;
    async fn graphql_request<T: DeserializeOwned + 'static>(&self, token: &str, query: &str, variables: Option<serde_json::Value>) -> Result<GraphQLResponse<T>, Box<dyn std::error::Error>>;
}

pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        ReqwestClient {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ApiClient for ReqwestClient {
    async fn login_request(&self, request: &GraphQLRequest) -> Result<GraphQLResponse<crate::models::LoginData>, Box<dyn std::error::Error>> {
        let response = self.client
            .post("https://weightxreps.net/api/graphql")
            .json(request)
            .send()
            .await?;
        let body: GraphQLResponse<crate::models::LoginData> = response.json().await?;
        Ok(body)
    }

    async fn graphql_request<T: DeserializeOwned + 'static>(&self, token: &str, query: &str, variables: Option<serde_json::Value>) -> Result<GraphQLResponse<T>, Box<dyn std::error::Error>> {
        let request_body = if let Some(vars) = variables {
            serde_json::json!({ "query": query, "variables": vars })
        } else {
            serde_json::json!({ "query": query })
        };
        let response = self.client
            .post("https://weightxreps.net/api/graphql")
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_body)
            .send()
            .await?;
        let body: GraphQLResponse<T> = response.json().await?;
        Ok(body)
    }
}

pub async fn login_request<C: ApiClient>(client: &C, request: &GraphQLRequest) -> Result<GraphQLResponse<crate::models::LoginData>, Box<dyn std::error::Error>> {
    client.login_request(request).await
}

pub async fn graphql_request<T: DeserializeOwned + 'static, C: ApiClient>(client: &C, token: &str, query: &str, variables: Option<serde_json::Value>) -> Result<GraphQLResponse<T>, Box<dyn std::error::Error>> {
    client.graphql_request(token, query, variables).await
}

#[allow(dead_code)]
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