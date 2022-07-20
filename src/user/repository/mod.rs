mod create_user;
mod delete_user;
mod find_all_users;
mod find_user_by_id;
mod find_user_by_name;
mod update_user;

use uuid::Uuid;

use super::{entities::User, service::PageInfo};

#[derive(Debug, Clone)]
pub struct Repository {}

impl Repository {
    pub fn new() -> Repository {
        Repository {}
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Pagination {
    pub first: Option<i32>,
    pub after: Option<Uuid>,
    pub last: Option<i32>,
    pub before: Option<Uuid>,
    pub is_total_count: bool,
    pub is_has_next_page: bool,
    pub is_has_previous_page: bool,
}

#[derive(Debug)]
pub struct UserConnection {
    pub edges: Vec<User>,
    pub page_info: PageInfo,
    pub total_count: Option<i64>,
}
