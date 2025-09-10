
pub struct AppState {
    pub ready: bool,
}

impl Default for AppState {
    fn default() -> Self { Self { ready: false } }
}