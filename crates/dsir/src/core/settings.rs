use std::sync::{Arc, LazyLock, RwLock};

use super::LM;
use crate::adapter::ChatAdapter;

pub struct Settings {
    pub lm: Arc<LM>,
    pub adapter: ChatAdapter,
}

impl Settings {
    pub fn new(lm: LM, adapter: ChatAdapter) -> Self {
        Self {
            lm: Arc::new(lm),
            adapter,
        }
    }
}

pub static GLOBAL_SETTINGS: LazyLock<RwLock<Option<Settings>>> =
    LazyLock::new(|| RwLock::new(None));

pub fn get_lm() -> Arc<LM> {
    Arc::clone(&GLOBAL_SETTINGS.read().unwrap().as_ref().unwrap().lm)
}

/// Returns the adapter configured via [`configure`], or a default [`ChatAdapter`].
pub fn chat_adapter() -> ChatAdapter {
    GLOBAL_SETTINGS
        .read()
        .unwrap()
        .as_ref()
        .map(|settings| settings.adapter.clone())
        .unwrap_or_default()
}

pub fn configure(lm: LM, adapter: ChatAdapter) {
    let settings = Settings::new(lm, adapter);
    *GLOBAL_SETTINGS.write().unwrap() = Some(settings);
}
