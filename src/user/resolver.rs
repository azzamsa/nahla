use std::sync::Arc;

use async_graphql::{Context, Error, FieldResult, Object};
use uuid::Uuid;

use super::model::input;
use super::model::User;
use crate::context::ServerContext;
use crate::user::scalar::Id;

#[derive(Default)]
pub struct UserQuery;

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserQuery {
    pub async fn users(&self, ctx: &Context<'_>) -> FieldResult<Vec<User>> {
        let server_ctx = ctx.data::<Arc<ServerContext>>()?;

        let result = server_ctx.user_service.find_users().await;
        match result {
            Ok(users) => Ok(users
                .into_iter()
                .map(|user| User {
                    id: Some(user.id),
                    created_at: Some(user.created_at),

                    name: user.name,
                    full_name: user.full_name,

                    unique_hash: Some("my unique hash!".to_string()),
                })
                .collect()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }
    pub async fn user(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<User> {
        let server_ctx = ctx.data::<Arc<ServerContext>>()?;

        let result = server_ctx.user_service.find_user(id).await;
        match result {
            Ok(res) => Ok(res.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }
}

#[Object]
impl UserMutation {
    pub async fn create_user(
        &self,
        ctx: &Context<'_>,
        input: input::CreateUserInput,
    ) -> FieldResult<User> {
        let server_ctx = ctx.data::<Arc<ServerContext>>()?;

        let result = server_ctx.user_service.create_user(input.into()).await;
        match result {
            Ok(res) => Ok(res.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }
    pub async fn update_user(
        &self,
        ctx: &Context<'_>,
        input: input::UpdateUserInput,
    ) -> FieldResult<User> {
        let server_ctx = ctx.data::<Arc<ServerContext>>()?;

        let result = server_ctx.user_service.update_user(input.into()).await;
        match result {
            Ok(res) => Ok(res.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }
    pub async fn delete_user(&self, ctx: &Context<'_>, id: Id) -> FieldResult<User> {
        let server_ctx = ctx.data::<Arc<ServerContext>>()?;

        let result = server_ctx.user_service.delete_user(id).await;
        match result {
            Ok(res) => Ok(res.into()),
            Err(err) => Err(Error::new(err.to_string())),
        }
    }
}