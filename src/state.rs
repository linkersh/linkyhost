use tokio_util::sync::CancellationToken;

use crate::db::HostDB;

pub struct AppState {
    pub db: HostDB,
    pub cancel: CancellationToken
}
