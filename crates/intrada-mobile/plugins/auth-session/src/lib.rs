use serde::{Deserialize, Serialize};
#[cfg(target_os = "ios")]
use tauri::Manager;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Runtime,
};

#[cfg(target_os = "ios")]
mod mobile;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("auth-session: {0}")]
    Bridge(String),
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(self.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenArgs {
    pub url: String,
    pub callback_scheme: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenResult {
    pub callback_url: String,
}

#[tauri::command]
async fn open<R: Runtime>(app: AppHandle<R>, args: OpenArgs) -> Result<OpenResult> {
    #[cfg(target_os = "ios")]
    {
        let state = app.state::<mobile::AuthSession<R>>();
        state.open(args)
    }
    #[cfg(not(target_os = "ios"))]
    {
        let _ = (app, args);
        Err(Error::Bridge(
            "ASWebAuthenticationSession is iOS-only".into(),
        ))
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("auth-session")
        .invoke_handler(tauri::generate_handler![open])
        .setup(|app, _api| {
            #[cfg(target_os = "ios")]
            {
                let handle = mobile::init(app, _api)?;
                app.manage(handle);
            }
            #[cfg(not(target_os = "ios"))]
            {
                let _ = app;
            }
            Ok(())
        })
        .build()
}
