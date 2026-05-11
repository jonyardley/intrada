use leptos::prelude::*;

/// Size variant for [`BrandMark`].
#[derive(Clone, Copy, Default)]
pub enum BrandMarkSize {
    /// 20px icon, `text-lg` wordmark — for headers and tight chrome.
    #[default]
    Sm,
    /// 24px icon, `text-lg` wordmark — for login / focused brand surfaces.
    Md,
    /// 28px icon, `page-title` wordmark — for loading splash / hero surfaces.
    Lg,
}

/// Brand-mark: heroicons "musical-note" SVG + "Intrada" wordmark.
///
/// Renders an inline `<span>` with the icon + text — the caller wraps it
/// with `<A>` for nav links or `<div>` for static contexts.
///
/// Replaces hand-rolled brand markup in welcome.rs, login.rs, app_header.rs,
/// and the auth-loading screen (#541). Until a final glyph + sizing token
/// set is locked in design, this is the single source of truth for the
/// brand surface; edits propagate to every call site.
#[component]
pub fn BrandMark(#[prop(optional)] size: BrandMarkSize) -> impl IntoView {
    let (icon_class, wordmark_class) = match size {
        BrandMarkSize::Sm => (
            "w-5 h-5 text-accent",
            "text-lg font-bold text-primary font-heading",
        ),
        BrandMarkSize::Md => (
            "w-6 h-6 text-accent",
            "text-lg font-bold text-primary font-heading",
        ),
        BrandMarkSize::Lg => ("w-7 h-7 text-accent", "page-title"),
    };

    // `brand-mark` is a stable selector kept for future styling /
    // test hooks — no CSS rules attached today.
    view! {
        <span class="brand-mark inline-flex items-center gap-2.5">
            <svg class=icon_class viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
            </svg>
            <span class=wordmark_class>"Intrada"</span>
        </span>
    }
}
