use crate::{
    database,
    router::hello::state::{HelloState, get_hello_state},
    utils::errors::AppError,
    ws::{self, state::WsState},
};
use axum::extract::FromRef;
use sea_orm::DatabaseConnection;
use state_fullstack_context_macro::FromFullstackContextRef;

#[derive(Clone, FromRef)]
pub struct AppDatabase(pub DatabaseConnection);

// newtype패턴을 이용해서 고아규칙을 회피하며, 기존라우터에서 사용하는 state를 제공함으로서
#[derive(Clone, FromFullstackContextRef)]
pub struct AppExtension {
    pub db: AppDatabase,
    pub ws: WsState,
    pub hello: HelloState,
}

impl AppExtension {
    pub async fn init() -> Result<Self, AppError> {
        let db = AppDatabase(database::init_db().await?);
        Ok(AppExtension {
            ws: ws::state::init_state(),
            hello: get_hello_state(db.0.clone()),
            db,
        })
    }
}
