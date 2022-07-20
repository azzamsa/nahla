use super::{Pagination, Service};
use crate::{
    errors::{
        core::Error::{
            MissingFirstAndLastPaginationArguments, PassedFirstAndLastPaginationArguments,
        },
        Error,
    },
    relay::Base64Cursor,
    user::{
        model::{UserConnection, UserEdge},
        repository,
    },
};

impl Service {
    pub async fn find_users(&self, pagination: Pagination) -> Result<UserConnection, Error> {
        let (first, after, last, before) = (
            pagination.first,
            pagination.after,
            pagination.last,
            pagination.before,
        );

        match (first, last) {
            (None, None) => return Err(MissingFirstAndLastPaginationArguments.into()),
            (Some(_), Some(_)) => return Err(PassedFirstAndLastPaginationArguments.into()),
            (Some(_first), None) => (Some(first), None),
            (None, Some(last)) => (None, Some(last)),
        };

        let (after_uuid, before_uuid) = match (after, before) {
            (None, None) => (None, None),
            (Some(after), Some(before)) => (
                Some(Base64Cursor::decode(&after)?.into()),
                Some(Base64Cursor::decode(&before)?.into()),
            ),
            (Some(after), None) => (Some(Base64Cursor::decode(&after)?.into()), None),
            (None, Some(before)) => (None, Some(Base64Cursor::decode(&before)?.into())),
        };

        let pagination_db = repository::Pagination {
            first,
            after: after_uuid,
            last,
            before: before_uuid,
            is_total_count: pagination.is_total_count,
            is_has_next_page: pagination.is_has_next_page,
            is_has_previous_page: pagination.is_has_previous_page,
        };

        let user_connection_db = self.repo.find_all_users(&self.db, pagination_db).await?;

        let user_edges: Vec<UserEdge> = user_connection_db
            .edges
            .into_iter()
            .map(|user| user.into())
            .collect();
        let user_connection = UserConnection {
            edges: user_edges,
            page_info: user_connection_db.page_info.into(),
            total_count: user_connection_db.total_count,
        };

        Ok(user_connection)
    }
}
