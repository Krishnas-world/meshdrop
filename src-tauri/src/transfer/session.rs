use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub struct TransferSession {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_path: String,
}

pub static SESSIONS: OnceLock<
    Mutex<HashMap<String, TransferSession>>
> = OnceLock::new();

pub fn sessions()
    -> &'static Mutex<HashMap<String, TransferSession>>
{
    SESSIONS.get_or_init(|| {
        Mutex::new(HashMap::new())
    })
}