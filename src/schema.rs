use async_graphql::{EmptySubscription, MergedObject, Schema};

use crate::health::resolver::HealthQuery;
use crate::meta::resolver::MetaQuery;
use crate::user::resolver::{UserMutation, UserQuery};

#[derive(MergedObject, Default)]
pub struct Query(MetaQuery, UserQuery, HealthQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(UserMutation);

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;
