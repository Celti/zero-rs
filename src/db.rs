use crate::model::Item;
use crate::schema::*;

use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error::NotFound;
use diesel::sql_types::{BigInt, Text};
use warp::{reject, Rejection};

type ServerResult<T> = Result<T, Rejection>;

sql_function!(fn nextval(x: Text) -> BigInt);

pub struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn connect(url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = Pool::new(manager).expect("database connection pool");

        Self { pool }
    }

    fn get(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.clone().get().expect("database connection")
    }

    pub fn get_next_id(&self) -> ServerResult<i64> {
        diesel::select(nextval("items_id_seq"))
            .get_result(&self.get())
            .map_err(|_| reject::server_error())
    }

    pub fn add_item(&self, item: &Item) -> ServerResult<Item> {
        diesel::insert_into(items::table)
            .values(item)
            .get_result(&self.get())
            .map_err(|_| reject::server_error())
    }

    pub fn delete_item(&self, item: &Item) -> ServerResult<Item> {
        diesel::delete(item)
            .get_result(&self.get())
            .map_err(|_| reject::server_error())
    }

    pub fn get_item(&self, id: &str) -> ServerResult<Item> {
        items::table
            .filter(items::digest.eq(&id[1..]))
            .or_filter(items::label.eq(&id).and(items::private.eq(false)))
            .first(&self.get())
            .map_err(|e| match e {
                NotFound => warp::reject::not_found(),
                _ => warp::reject::server_error(),
            })
    }

    pub fn delete_item_by_digest(&self, id: &str) -> ServerResult<Item> {
        diesel::delete(items::table.filter(items::digest.eq(&id[1..])))
            .get_result(&self.get())
            .map_err(|e| match e {
                NotFound => warp::reject::not_found(),
                _ => warp::reject::server_error(),
            })
    }

    pub fn update_item(&self, p: &Item) -> ServerResult<Item> {
        p.save_changes(&self.get())
            .map_err(|_| reject::server_error())
    }

    pub fn sunset_items(&self) -> QueryResult<usize> {
        diesel::delete(items::table.filter(items::sunset.le(Utc::now()))).execute(&self.get())
    }
}
