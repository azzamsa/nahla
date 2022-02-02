use std::sync::Arc;

use anyhow::{Context, Result};
use async_graphql::{EmptySubscription, Schema};
use cynic::MutationBuilder;
use nahla::config::Config;
use nahla::context::ServerContext;
use nahla::routes::graphql_handler;
use nahla::schema::{Mutation, Query};
use nahla::{db, health, meta, user};
use poem::{test::TestClient, Route};
use serde_json::{from_str, Value};

use super::graphql::add;
use super::schema::CreateUserResponse;
use crate::user::graphql::update;

#[tokio::test]
async fn duplicate_username_create() -> Result<()> {
    // Setup app
    let config = Arc::new(Config::load()?);
    let db = db::connect(&config.database).await?;

    let user_service = Arc::new(user::Service::new(db.clone()));
    let meta_service = Arc::new(meta::Service::new());
    let health_service = Arc::new(health::Service::new());

    let server_context = Arc::new(ServerContext {
        user_service,
        meta_service,
        health_service,
    });

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(Arc::clone(&server_context))
        .finish();

    // Test
    let app = Route::new().at("/", graphql_handler);
    let client = TestClient::new(app);

    //
    // Create User
    //

    let args = add::CreateUserInput {
        name: "khawa-duplicate-create".to_string(),
        full_name: Some("Abu Musa Al-Khawarizmi".to_string()),
    };
    let query = add::UserMutation::build(&args);

    let resp = client
        .post("/")
        .data(schema.clone())
        .body_json(&query)
        .send()
        .await;
    resp.assert_status_is_ok();

    //
    // Create next user with the same name
    //

    let args = add::CreateUserInput {
        name: "khawa-duplicate-create".to_string(),
        full_name: None,
    };
    let query = add::UserMutation::build(&args);

    let resp = client
        .post("/")
        .data(schema.clone())
        .body_json(&query)
        .send()
        .await;
    let resp_str = resp.into_body().into_string().await?;

    let body: Value = from_str(&resp_str).context("failed to deserialize response")?;
    let error_message = &body["errors"][0]["message"];
    assert_eq!(error_message, "username is already in use");

    Ok(())
}

#[tokio::test]
async fn duplicate_username_update() -> Result<()> {
    // Setup app
    let config = Arc::new(Config::load()?);
    let db = db::connect(&config.database).await?;

    let user_service = Arc::new(user::Service::new(db.clone()));
    let meta_service = Arc::new(meta::Service::new());
    let health_service = Arc::new(health::Service::new());

    let server_context = Arc::new(ServerContext {
        user_service,
        meta_service,
        health_service,
    });

    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(Arc::clone(&server_context))
        .finish();

    // Test
    let app = Route::new().at("/", graphql_handler);
    let client = TestClient::new(app);

    //
    // Create User
    //

    let args = add::CreateUserInput {
        name: "khawa-duplicate-update".to_string(),
        full_name: Some("Abu Musa Al-Khawarizmi".to_string()),
    };
    let query = add::UserMutation::build(&args);

    let resp = client
        .post("/")
        .data(schema.clone())
        .body_json(&query)
        .send()
        .await;
    resp.assert_status_is_ok();

    //
    // Create second user
    //

    let args = add::CreateUserInput {
        name: "khawa-duplicate-update-1".to_string(),
        full_name: Some("Abu Musa Al-Khawarizmi".to_string()),
    };
    let query = add::UserMutation::build(&args);

    let resp = client
        .post("/")
        .data(schema.clone())
        .body_json(&query)
        .send()
        .await;
    resp.assert_status_is_ok();

    let resp_str = resp.into_body().into_string().await?;
    let user_response: CreateUserResponse = from_str(&resp_str)?;
    let user_id = user_response.data.create_user.id;

    //
    // Update second user to the same name as first user
    //

    let user_id = update::Uuid(user_id.to_string());
    let args = update::UpdateUserInput {
        id: user_id,
        name: "khawa-duplicate-update".to_string(),
        full_name: None,
    };
    let query = update::UserMutation::build(&args);

    let resp = client
        .post("/")
        .data(schema.clone())
        .body_json(&query)
        .send()
        .await;
    let resp_str = resp.into_body().into_string().await?;
    let body: Value = from_str(&resp_str).context("failed to deserialize response")?;
    let error_message = &body["errors"][0]["message"];

    assert_eq!(error_message, "username is already in use");

    Ok(())
}
