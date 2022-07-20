use std::sync::Arc;

use async_graphql::{Context, Error, FieldResult, Object};
use uuid::Uuid;

use super::{
    model::{input, User, UserConnection},
    service::Pagination,
};
use crate::{context::ServerContext, user::scalar::Id};

#[derive(Default)]
pub struct UserQuery;

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserQuery {
    pub async fn users(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> FieldResult<UserConnection> {
        let server_ctx = ctx.data::<Arc<ServerContext>>()?;
        let is_total_count = ctx.look_ahead().field("totalCount").exists();
        let is_has_next_page = ctx
            .look_ahead()
            .field("pageInfo")
            .field("hasNextPage")
            .exists();
        let is_has_previous_page = ctx
            .look_ahead()
            .field("pageInfo")
            .field("hasPreviousPage")
            .exists();

        let pagination_opts = Pagination {
            after,
            before,
            first,
            last,
            is_total_count,
            is_has_next_page,
            is_has_previous_page,
        };

        let result = server_ctx.user_service.find_users(pagination_opts).await;
        match result {
            Ok(users) => Ok(users),
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
