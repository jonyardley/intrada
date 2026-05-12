use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{OpenArgs, OpenResult};

tauri::ios_plugin_binding!(init_plugin_auth_session);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<AuthSession<R>> {
    let handle = api
        .register_ios_plugin(init_plugin_auth_session)
        .map_err(|e| crate::Error::Bridge(e.to_string()))?;
    Ok(AuthSession(handle))
}

pub struct AuthSession<R: Runtime>(pub PluginHandle<R>);

impl<R: Runtime> AuthSession<R> {
    pub fn open(&self, args: OpenArgs) -> crate::Result<OpenResult> {
        self.0
            .run_mobile_plugin::<OpenResult>("open", args)
            .map_err(|e| crate::Error::Bridge(e.to_string()))
    }
}
