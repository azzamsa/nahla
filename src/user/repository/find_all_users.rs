use sqlx::{self, Row};

use super::{Pagination, Repository, UserConnection};
use crate::{
    db::Queryer,
    errors::core::Error,
    relay::Base64Cursor,
    user::{entities, service::PageInfo},
};

impl Repository {
    pub async fn find_all_users<'c, C: Queryer<'c> + Copy>(
        &self,
        db: C,
        pagination: Pagination,
    ) -> Result<UserConnection, Error> {
        let default_page_size = 10;
        let (first, after, last, before) = (
            pagination.first,
            pagination.after,
            pagination.last,
            pagination.before,
        );

        let mut query: String = "select * from user_".to_string();
        let mut has_next_query: String = String::new();

        let mut has_next_page: Option<bool> = Some(false);
        let mut has_previous_page: Option<bool> = Some(false);

        match (first, after, last, before) {
            // First
            (Some(first), None, None, None) => {
                query = format!("{query} order by id asc limit {}", first);
                    has_next_query = format!(
                    r#"select count(*) > {first} from
                     ( select "id" from user_ order by id asc limit {limit} )
                   as data"#,
                    limit = first + 1
                );


            }
            // First & after,
            (Some(first), Some(after), None, None) => {
                query = format!("{query} where id > '{after}' order by id asc limit {first}");
                has_next_query = format!(
                    r#"select count(*) > {first} from
                     ( select "id" from user_ where id > '{after}' order by id asc limit {limit} )
                   as data"#,
                    limit = first + 1
                );


            }
            // Last
            (None, None, Some(last), None) => {
                query = format!(
                    "select * from ( select * from user_ order by id desc limit {limit} ) as data order by id asc",
                    limit = last + 1
                );
            }
            // Last & before
            (None, None, Some(last), Some(before)) => {
                query = format!("select * from ( select * from user_ where id < '{before}' order by id desc limit {limit} ) as data order by id asc;", limit = last + 1)
            } // Default page size
            _ => query = format!("{query} limit {}", default_page_size),
        };

        let mut rows = match sqlx::query_as::<_, entities::User>(&query)
            .fetch_all(db)
            .await
        {
            Err(err) => {
                log::error!("finding users: {}", &err);
                return Err(err.into());
            }
            Ok(res) => res,
        };

        //
        // total count
        //
        let total_count_query = "select count(*) as exact_count from  user_";
        let total_count = match pagination.is_total_count {
            true => match sqlx::query(total_count_query).fetch_one(db).await {
                Err(err) => {
                    log::error!("counting users: {}", &err);
                    return Err(err.into());
                }
                Ok(row) => Some(row.get(0)),
            },
            false => None,
        };
        //
        // has_next_page
        //
        match pagination.is_has_next_page {
            true => {
                if let Some(_first) = first {
                    has_next_page = match sqlx::query(&has_next_query).fetch_one(db).await {
                        Err(err) => {
                            log::error!("calculating has_next in users: {}", &err);
                            return Err(err.into());
                        }
                        Ok(row) => Some(row.get(0)),
                    }
                };
            }
            false => {
                has_next_page = None;
            }
        }
        //
        // has_previous_page
        //
        match pagination.is_has_previous_page {
            true => {
                if let Some(last) = last {
                    log::debug!("rows length: {}. last: {}", rows.len(), last);
                    has_previous_page = Some(rows.len() > last.try_into()?);

                    // The real value start from index 1. The 0 index only act as a sign for `has_previous_page`
                    rows = match has_previous_page {
                        Some(true) => rows[1..rows.len()].to_vec(),
                        Some(false) => rows,
                        // TODO should return None
                        None => rows,
                    }
                };
            }
            false => {
                has_previous_page = None;
            }
        }

        let (start_cursor, end_cursor) = if !rows.is_empty() {
            let start_cursor = Base64Cursor::new(rows[0].id).encode();
            let end_cursor = Base64Cursor::new(rows[rows.len() - 1].id).encode();
            (Some(start_cursor), Some(end_cursor))
        } else {
            (None, None)
        };

        let page_info = PageInfo {
            has_next_page,
            has_previous_page,
            start_cursor,
            end_cursor,
        };

        let user_connection = UserConnection {
            edges: rows,
            page_info,
            total_count,
        };
        Ok(user_connection)
    }
}
