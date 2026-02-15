use std::sync::Arc;

use libsql::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub allowed_origin: String,
}

impl AppState {
    pub fn new(db: Database, allowed_origin: String) -> Self {
        Self {
            db: Arc::new(db),
            allowed_origin,
        }
    }
}
