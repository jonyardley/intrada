//! Rust/WASM bindings for the Clerk JS SDK via `window.__intrada_auth`.
//!
//! All functions call into the JavaScript helpers defined in `index.html`.
//! In E2E tests, `window.__intrada_auth` is stubbed before page load.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
    export function js_init_clerk(key) {
        if (window.__intrada_auth) {
            window.__intrada_auth.init(key);
        }
    }
    export function js_is_signed_in() {
        if (window.__intrada_auth) {
            return window.__intrada_auth.isSignedIn();
        }
        return false;
    }
    export function js_init_failed() {
        if (window.__intrada_auth && window.__intrada_auth.initFailed) {
            return window.__intrada_auth.initFailed();
        }
        return false;
    }
    export async function js_get_token() {
        if (window.__intrada_auth) {
            return await window.__intrada_auth.getToken();
        }
        return null;
    }
    export function js_get_user_id() {
        if (window.__intrada_auth) {
            return window.__intrada_auth.getUserId();
        }
        return null;
    }
    export function js_get_user_email() {
        if (window.__intrada_auth && window.__intrada_auth.getUserEmail) {
            return window.__intrada_auth.getUserEmail();
        }
        return null;
    }
    export async function js_sign_out() {
        if (window.__intrada_auth) {
            await window.__intrada_auth.signOut();
        }
    }
    export async function js_sign_in_with_google() {
        if (window.__intrada_auth) {
            await window.__intrada_auth.signInWithGoogle();
        }
    }
    export function js_add_auth_listener(callback) {
        if (window.__intrada_auth) {
            window.__intrada_auth.addListener(function() {
                callback();
            });
        }
    }
")]
extern "C" {
    fn js_init_clerk(key: &str);
    fn js_is_signed_in() -> bool;
    fn js_init_failed() -> bool;
    async fn js_get_token() -> JsValue;
    fn js_get_user_id() -> JsValue;
    fn js_get_user_email() -> JsValue;
    async fn js_sign_out();
    async fn js_sign_in_with_google();
    fn js_add_auth_listener(callback: &Closure<dyn Fn()>);
}

/// Initialize Clerk with the publishable key.
pub fn init_clerk() {
    let key = option_env!("CLERK_PUBLISHABLE_KEY").unwrap_or("");
    if !key.is_empty() {
        js_init_clerk(key);
    }
}

/// Check whether the current user is signed in.
pub fn is_signed_in() -> bool {
    js_is_signed_in()
}

/// Check whether Clerk initialization failed (bad key, wrong domain, network error).
/// When true, the app should bypass the auth gate.
pub fn init_failed() -> bool {
    js_init_failed()
}

/// Get the current JWT token for API requests. Returns `None` if not signed in.
pub async fn get_auth_token() -> Option<String> {
    let val = js_get_token().await;
    val.as_string()
}

/// Get the current user's Clerk user ID. Returns `None` if not signed in.
pub fn get_user_id() -> Option<String> {
    let val = js_get_user_id();
    val.as_string()
}

/// Get the current user's primary email address. Returns `None` if not
/// signed in or if the Clerk user has no email on file.
pub fn email() -> Option<String> {
    let val = js_get_user_email();
    val.as_string()
}

/// Sign out the current user.
pub async fn sign_out() {
    js_sign_out().await;
}

/// Redirect to Google OAuth sign-in.
pub async fn sign_in_with_google() {
    js_sign_in_with_google().await;
}

/// Register a listener for auth state changes (sign-in/sign-out).
pub fn add_auth_listener(callback: &Closure<dyn Fn()>) {
    js_add_auth_listener(callback);
}
