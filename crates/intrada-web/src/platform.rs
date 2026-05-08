/// Returns true when running inside the Tauri iOS WebView.
///
/// Reads the `data-platform="ios"` attribute set on `<html>` by the inline
/// `index.html` script when `location.protocol === 'tauri:'`. The attribute
/// is set before first paint, so this check is reliable from any view's
/// on-mount logic — no race against the DOM coming up.
///
/// Returns false in regular browsers (where the protocol is http/https) and
/// in any environment without a window/document, so it's safe to call
/// outside the WebView (e.g. wasm-bindgen tests).
pub fn is_ios() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|e| e.get_attribute("data-platform"))
        .map(|p| p == "ios")
        .unwrap_or(false)
}
