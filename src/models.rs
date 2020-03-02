use crate::schema::users;
use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager},
    Queryable,
};
use std::env;
use std::sync::{Arc, RwLock};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbCon = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn create_db_pool(size: u32) -> DbPool {
    r2d2::Pool::builder()
        .max_size(size)
        .build(ConnectionManager::<PgConnection>::new(database_url()))
        .expect("failed to create db connection pool")
}

pub fn create_db_pools(size: u32) -> RoundRobin<DbPool> {
    RoundRobin::new(
        database_url()
            .split(',')
            .map(|url| {
                r2d2::Pool::builder()
                    .max_size(size)
                    .build(ConnectionManager::<PgConnection>::new(dbg!(url)))
                    .expect("failed to create db connection pool")
            })
            .collect(),
    )
}

#[derive(Debug, Clone, Queryable)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
}

pub struct RoundRobin<T> {
    items: Vec<T>,
    current_index: Arc<RwLock<usize>>,
}

impl<T> RoundRobin<T> {
    pub fn new(items: Vec<T>) -> Self {
        RoundRobin {
            items: items,
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub fn next(&self) -> Option<&T> {
        let mut i = self.current_index.write().unwrap();
        let ret = self.items.get(*i);
        *i = (*i + 1) % self.items.len();
        ret
    }
}
