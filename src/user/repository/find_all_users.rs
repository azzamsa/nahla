use sqlx::{self, Row};
use uuid::Uuid;

use super::Repository;
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
        first: Option<i32>,
        after: Option<Uuid>,
        last: Option<i32>,
        before: Option<Uuid>,
    ) -> Result<Vec<entities::User>, Error> {
        let default_page_size = 10;
        let mut query: String = "select * from user_".to_string();

        match (first, after, last, before) {
            // First
            (Some(first), None, None, None) => {
                query = format!("{query} order by id asc limit {}", first);
            }
            // First & after,
            (Some(first), Some(after), None, None) => {
                query = format!("{query} where id > '{after}' order by id asc limit {first}");
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
            }
            // Default page size
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

        let has_previous_page = self.has_previous_page(&rows, last).await?;
        if last.is_some() {
            // The real value start from index 1. The 0 index only act as a sign for `has_previous_page`
            rows = if has_previous_page {
                rows[1..rows.len()].to_vec()
            } else {
                rows
            }
        };
        Ok(rows)
    }
    pub async fn has_previous_page(
        &self,
        rows: &Vec<entities::User>,
        last: Option<i32>,
    ) -> Result<bool, Error> {
        let mut has_previous_page: bool = false;
        if let Some(last) = last {
            log::debug!("rows length: {}. last: {}", rows.len(), last);
            has_previous_page = rows.len() > last.try_into()?;
        };
        Ok(has_previous_page)
    }
    pub async fn find_page_info<'c, C: Queryer<'c> + Copy>(
        &self,
        db: C,
        rows: &Vec<entities::User>,
        first: Option<i32>,
        after: Option<Uuid>,
        last: Option<i32>,
        before: Option<Uuid>,
    ) -> Result<PageInfo, Error> {
        let mut has_next_query: String = String::new();
        let mut has_next_page: bool = false;

        match (first, after, last, before) {
            // First
            (Some(first), None, None, None) => {
                has_next_query = format!(
                    r#"select count(*) > {first} from
                     ( select "id" from user_ order by id asc limit {limit} )
                   as data"#,
                    limit = first + 1
                );
            }
            // First & after,
            (Some(first), Some(after), None, None) => {
                has_next_query = format!(
                    r#"select count(*) > {first} from
                     ( select "id" from user_ where id > '{after}' order by id asc limit {limit} )
                   as data"#,
                    limit = first + 1
                );
            }
            _ => (),
        };

        //
        // has_next query
        //
        if let Some(_first) = first {
            has_next_page = match sqlx::query(&has_next_query).fetch_one(db).await {
                Err(err) => {
                    log::error!("calculating has_next in users: {}", &err);
                    return Err(err.into());
                }
                Ok(row) => row.get(0),
            };
        };

        let (start_cursor, end_cursor) = if !rows.is_empty() {
            let start_cursor = Base64Cursor::new(rows[0].id).encode();
            let end_cursor = Base64Cursor::new(rows[rows.len() - 1].id).encode();
            (Some(start_cursor), Some(end_cursor))
        } else {
            (None, None)
        };

        let has_previous_page = self.has_previous_page(rows, last).await?;
        let page_info = PageInfo {
            has_next_page,
            has_previous_page,
            start_cursor,
            end_cursor,
        };

        Ok(page_info)
    }
}
