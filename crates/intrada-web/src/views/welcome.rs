use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::app::AuthState;
use crate::components::{Button, ButtonVariant};
use intrada_web::js_bridge;
use intrada_web::platform::is_ios;

/// Public marketing homepage at `/`.
///
/// Pencil ref: `fWsUw` (desktop) and `vFOGv` (mobile-web).
///
/// Renders immediately — does not wait for Clerk to initialise. Auth-state-
/// based redirects fire reactively via the Effect below as soon as Clerk
/// resolves:
/// - Authed users → redirect to /library.
/// - Unauthed iOS users (Tauri WebView) → redirect to /login. They already
///   downloaded the app; the marketing pitch is redundant. Held until
///   `auth_loading=false` so we know they're really unauthed (vs. Clerk
///   still loading their session).
/// - Anyone else → render the marketing page.
///
/// Sub-sections live as private components in this file. Phone-frame
/// graphics in the hero use a small reusable `PhoneFrame` shell with
/// hand-built faux content matching the Pencil mocks.
#[component]
pub fn WelcomeView() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let on_ios = is_ios();

    Effect::new(move |_| {
        if auth.is_authenticated.get() {
            let navigate = use_navigate();
            navigate(
                "/library",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        } else if !auth.auth_loading.get() && on_ios {
            let navigate = use_navigate();
            navigate(
                "/login",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    view! {
        <div class="relative z-0 min-h-screen text-primary">
            <WelcomeNav />
            <WelcomeHero />
            <WelcomePillars />
            <WelcomeDifferentiators />
            <WelcomeFeature
                kicker="PLAN".to_string()
                title="The library that\nremembers for you.".to_string()
                description="Half of what your teacher gives you disappears within a week. Intrada catches it — tag by composer, key, or tempo, group what you actually run into reusable routines, and connect everything to the goals you're working towards.".to_string()
                bullets=vec![
                    "Pieces, exercises, and patterns in one place".to_string(),
                    "Reusable routines for the warm-ups and blocks you run every week".to_string(),
                    "Goals that turn the library into a path".to_string(),
                ]
                reverse=false
                mock=Box::new(|| view! { <LibraryMock /> }.into_any())
            />
            <WelcomeFeature
                kicker="PRACTICE".to_string()
                title="One decision:\nstart.".to_string()
                description="Choosing what to practise is the silent killer of a session. Intrada plans the work for the time you've got — you tap start and play. Each item gets a timer and a quick rating, then on to the next. Big numbers, big buttons, no chrome.".to_string()
                bullets=vec![
                    "A session ready every time you sit down".to_string(),
                    "Resume right where you left off".to_string(),
                    "Adjust duration and focus mid-session without losing your place".to_string(),
                ]
                reverse=true
                mock=Box::new(|| view! { <PracticeMock /> }.into_any())
            />
            <WelcomeFeature
                kicker="TRACK".to_string()
                title="Progress you\ncan't yet feel.".to_string()
                description="Music improves between sessions, not within them — so the work feels invisible while it's happening. Intrada turns weeks of mastery ratings, time, and tempo into evidence you can see today, before your ears catch up.".to_string()
                bullets=vec![
                    "Mastery per piece, per key, across weeks".to_string(),
                    "Time, sessions, and where your attention actually went".to_string(),
                    "A welcome back when life happens — not a guilt trip".to_string(),
                ]
                reverse=false
                mock=Box::new(|| view! { <AnalyticsMock /> }.into_any())
            />
            <WelcomeFinalCta />
            <WelcomeFooter />
        </div>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Top nav — sticky, glass-chrome, brand + nav links + sign-in CTA
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomeNav() -> impl IntoView {
    let signing_in = RwSignal::new(false);
    let on_sign_in = Callback::new(move |_| {
        signing_in.set(true);
        leptos::task::spawn_local(async move {
            js_bridge::sign_in_with_google().await;
        });
    });

    view! {
        <header class="sticky top-0 z-40 glass-chrome border-b border-border-default">
            <div class="max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 py-4 flex items-center justify-between">
                <A href="/" attr:class="flex items-center gap-2.5 no-underline">
                    <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                    </svg>
                    <span class="text-lg font-bold text-primary font-heading">"Intrada"</span>
                </A>

                <nav class="hidden md:flex items-center gap-8">
                    <a href="#pillars" class="text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">"Features"</a>
                    <a href="#feature-plan" class="text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">"How it works"</a>
                    <a href="#feature-track" class="text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">"For musicians"</a>
                </nav>

                <div class="flex items-center gap-3">
                    <A href="/login" attr:class="hidden sm:inline-flex text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">
                        "Sign in"
                    </A>
                    <Button variant=ButtonVariant::Primary on_click=on_sign_in loading=signing_in.read_only()>
                        "Get started"
                    </Button>
                </div>
            </div>
        </header>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Hero — headline, sub, dual CTAs, 3-phone showcase
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomeHero() -> impl IntoView {
    let signing_in = RwSignal::new(false);
    let on_sign_in = Callback::new(move |_| {
        signing_in.set(true);
        leptos::task::spawn_local(async move {
            js_bridge::sign_in_with_google().await;
        });
    });

    view! {
        <section class="max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 pt-20 sm:pt-28 pb-12 text-center">
            // Beta badge
            <div class="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-surface-faint border border-border-default mb-8">
                <span class="w-1.5 h-1.5 rounded-full bg-accent"></span>
                <span class="text-xs font-semibold text-secondary">"Now in beta · iOS + Web"</span>
            </div>

            <h1 class="text-5xl sm:text-7xl lg:text-8xl font-bold text-primary mb-6 leading-[1.05] tracking-tight font-heading">
                "Practice with intent."
            </h1>

            <p class="text-base sm:text-lg lg:text-xl text-secondary max-w-2xl mx-auto leading-relaxed mb-10">
                "Music has one of the longest feedback loops of any skill. Intrada captures what you're working on, plans what to practise next, and surfaces the progress you can't yet hear."
            </p>

            <div class="flex flex-col sm:flex-row items-center justify-center gap-3 mb-20">
                <Button
                    variant=ButtonVariant::Primary
                    on_click=on_sign_in
                    loading=signing_in.read_only()
                >
                    "Sign in with Google"
                </Button>
                <a
                    href="#pillars"
                    class="inline-flex items-center justify-center gap-1.5 rounded-lg bg-surface-secondary border border-border-default px-3.5 py-2.5 text-sm font-medium text-label hover:bg-surface-hover motion-safe:transition-colors min-h-[44px] no-underline"
                >
                    "See how it works"
                </a>
            </div>

            // Phone showcase — 3 mocks, middle one slightly raised for visual rhythm
            <div class="flex items-end justify-center gap-4 sm:gap-6 flex-wrap">
                <div class="hidden lg:block">
                    <PhoneFrame><LibraryMock /></PhoneFrame>
                </div>
                <PhoneFrame><PracticeMock /></PhoneFrame>
                <div class="hidden lg:block">
                    <PhoneFrame><AnalyticsMock /></PhoneFrame>
                </div>
            </div>
        </section>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Three pillars — Plan / Practice / Track
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomePillars() -> impl IntoView {
    view! {
        <section id="pillars" class="max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 py-20 sm:py-24">
            <div class="text-center mb-16">
                <p class="text-xs font-bold tracking-[0.15em] text-accent-text mb-3">"THREE PILLARS"</p>
                <h2 class="text-3xl sm:text-5xl font-bold text-primary mb-5 leading-tight font-heading">
                    "A complete practice loop."
                </h2>
                <p class="text-base sm:text-lg text-secondary max-w-2xl mx-auto leading-relaxed">
                    "Built around the three things real practice actually needs — somewhere to keep the material, a plan for today, and evidence the work is paying off."
                </p>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <PillarCard
                    icon="library".to_string()
                    title="Plan".to_string()
                    description="Stop losing material. Every piece, exercise, voicing and lick lives in one place — captured once, surfaced when it matters.".to_string()
                />
                <PillarCard
                    icon="timer".to_string()
                    title="Practice".to_string()
                    description="Stop deciding what to practise. Tap start. Intrada plans the session, runs the timer, and gets out of the way.".to_string()
                />
                <PillarCard
                    icon="trending-up".to_string()
                    title="Track".to_string()
                    description="Stop wondering if it's working. Mastery, time, and per-piece progress — daily evidence that the work is paying off.".to_string()
                />
            </div>
        </section>
    }
}

#[component]
fn PillarCard(icon: String, title: String, description: String) -> impl IntoView {
    let icon_svg = match icon.as_str() {
        "library" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="w-6 h-6">
                <path d="M4 19.5v-15A2.5 2.5 0 016.5 2H20v20H6.5a2.5 2.5 0 01-2.5-2.5z M4 19.5A2.5 2.5 0 016.5 17H20"/>
            </svg>
        }.into_any(),
        "timer" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="w-6 h-6">
                <line x1="10" y1="2" x2="14" y2="2"/>
                <line x1="12" y1="14" x2="15" y2="11"/>
                <circle cx="12" cy="14" r="8"/>
            </svg>
        }.into_any(),
        "calendar" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="w-6 h-6">
                <rect x="3" y="5" width="18" height="16" rx="2"/>
                <line x1="3" y1="10" x2="21" y2="10"/>
                <line x1="8" y1="3" x2="8" y2="7"/>
                <line x1="16" y1="3" x2="16" y2="7"/>
            </svg>
        }.into_any(),
        "layers" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="w-6 h-6">
                <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                <path d="M2 17l10 5 10-5"/>
                <path d="M2 12l10 5 10-5"/>
            </svg>
        }.into_any(),
        "sparkles" => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="w-6 h-6">
                <path d="M12 3l1.8 6.2L20 11l-6.2 1.8L12 19l-1.8-6.2L4 11l6.2-1.8z"/>
                <path d="M19 3l.7 2.3L22 6l-2.3.7L19 9l-.7-2.3L16 6l2.3-.7z"/>
            </svg>
        }.into_any(),
        _ => view! {
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" class="w-6 h-6">
                <polyline points="22 7 13.5 15.5 8.5 10.5 2 17"/>
                <polyline points="16 7 22 7 22 13"/>
            </svg>
        }.into_any(),
    };

    view! {
        <div class="card p-8 flex flex-col items-start gap-4">
            <div class="w-12 h-12 rounded-surface bg-badge-piece-bg flex items-center justify-center text-accent-text">
                {icon_svg}
            </div>
            <h3 class="text-2xl font-bold text-primary font-heading">
                {title}
            </h3>
            <p class="text-base text-secondary leading-relaxed">{description}</p>
        </div>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Differentiators — sets up unique positioning vs. timer-and-streak apps
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomeDifferentiators() -> impl IntoView {
    view! {
        <section class="max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 py-20 sm:py-24">
            <div class="text-center mb-16">
                <p class="text-xs font-bold tracking-[0.15em] text-accent-text mb-3">"WHY INTRADA"</p>
                <h2 class="text-3xl sm:text-5xl font-bold text-primary mb-5 leading-tight font-heading">
                    "Built on what works."
                </h2>
                <p class="text-base sm:text-lg text-secondary max-w-2xl mx-auto leading-relaxed">
                    "Most practice apps measure attendance. Intrada is built on decades of research into how musicians actually learn — and how real progress hides until the data shows it."
                </p>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <PillarCard
                    icon="calendar".to_string()
                    title="Schedules itself around you".to_string()
                    description="Spaced repetition and interleaved practice are settled learning science. Intrada is being built around them — so weak items surface more often, work from six weeks ago won't silently decay, and every session fits the time you've got.".to_string()
                />
                <PillarCard
                    icon="layers".to_string()
                    title="Tracks the detail that matters".to_string()
                    description="Mastery per piece, per key, per tempo — not just total minutes. The fine grain is where invisible improvements live, and where the scheduler finds the work that needs you most.".to_string()
                />
                <PillarCard
                    icon="sparkles".to_string()
                    title="Designed for every mind".to_string()
                    description="One tap from open to playing. No streak counters, no shame. A calm interface that gets out of the way during practice. Built with ADHD and neurodivergent musicians in mind — and better for everyone.".to_string()
                />
            </div>
        </section>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Feature deep-dive — alternating text+mock sections
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomeFeature(
    kicker: String,
    title: String,
    description: String,
    bullets: Vec<String>,
    reverse: bool,
    mock: Box<dyn Fn() -> AnyView + Send + Sync>,
) -> impl IntoView {
    let layout_class = "max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 py-20 sm:py-24 grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-20 items-center";

    let text_order = if reverse { "lg:order-2" } else { "" };
    // Below the `lg:` breakpoint the grid collapses to a single column and
    // the mock would otherwise stretch to the full content max-width
    // (~1280px on tablet). Cap it at `max-w-md` and centre it so the rows
    // and stat cards keep proportions close to how they're sized inside
    // the hero PhoneFrame. At `lg:` and up the column itself constrains
    // the width, so we drop the cap.
    let mock_order = if reverse {
        "lg:order-1 w-full max-w-md mx-auto lg:max-w-none lg:mx-0"
    } else {
        "w-full max-w-md mx-auto lg:max-w-none lg:mx-0"
    };

    let id = format!("feature-{}", kicker.to_lowercase());

    view! {
        <section id=id class=layout_class>
            <div class=format!("flex flex-col items-start gap-6 {text_order}")>
                <p class="text-xs font-bold tracking-[0.15em] text-accent-text">{kicker}</p>
                <h2 class="text-3xl sm:text-5xl font-bold text-primary leading-tight whitespace-pre-line font-heading">
                    {title}
                </h2>
                <p class="text-base sm:text-lg text-secondary leading-relaxed">{description}</p>
                <ul class="flex flex-col gap-3 mt-2">
                    {bullets.into_iter().map(|b| view! {
                        <li class="flex items-center gap-3">
                            <svg class="w-5 h-5 text-accent-text shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                <circle cx="12" cy="12" r="10"/>
                                <polyline points="9 12 11 14 15 10"/>
                            </svg>
                            <span class="text-base text-secondary">{b}</span>
                        </li>
                    }).collect_view()}
                </ul>
            </div>
            <div class=format!("card p-5 sm:p-6 flex flex-col gap-3.5 {mock_order}")>
                {mock()}
            </div>
        </section>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Final CTA card
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomeFinalCta() -> impl IntoView {
    let signing_in = RwSignal::new(false);
    let on_sign_in = Callback::new(move |_| {
        signing_in.set(true);
        leptos::task::spawn_local(async move {
            js_bridge::sign_in_with_google().await;
        });
    });

    view! {
        <section class="max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 py-20 sm:py-32">
            <div class="card p-12 sm:p-16 flex flex-col items-center text-center">
                <h2 class="text-4xl sm:text-6xl font-bold text-primary mb-5 leading-tight font-heading">
                    "Let's practice."
                </h2>
                <p class="text-base sm:text-lg text-secondary mb-8 max-w-xl leading-relaxed">
                    "Sign in with Google. Free during beta — no credit card."
                </p>
                <Button
                    variant=ButtonVariant::Primary
                    on_click=on_sign_in
                    loading=signing_in.read_only()
                >
                    "Sign in with Google"
                </Button>
                <p class="text-faint text-sm mt-6">"Available on iOS and the web."</p>
            </div>
        </section>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Footer
// ════════════════════════════════════════════════════════════════════════

#[component]
fn WelcomeFooter() -> impl IntoView {
    view! {
        <footer class="max-w-7xl mx-auto px-6 sm:px-8 lg:px-12 py-10 border-t border-border-default flex flex-col sm:flex-row items-center justify-between gap-4">
            <div class="flex items-center gap-2.5">
                <svg class="w-4 h-4 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                </svg>
                <span class="text-base font-bold text-primary font-heading">"Intrada"</span>
            </div>
            <p class="text-sm text-muted">"© 2026 Intrada · Built for musicians who practice with intent."</p>
            <nav class="flex items-center gap-6">
                <a href="#" class="text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">"Privacy"</a>
                <a href="#" class="text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">"Terms"</a>
                <a href="#" class="text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors no-underline">"Contact"</a>
            </nav>
        </footer>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Phone frame — black bezel, Dynamic Island, gradient inner screen
// ════════════════════════════════════════════════════════════════════════

#[component]
fn PhoneFrame(children: Children) -> impl IntoView {
    view! {
        <div
            class="relative w-[280px] h-[580px] rounded-[48px] bg-black overflow-hidden shadow-2xl shrink-0"
            style="padding: 6px;"
        >
            // Inner screen — app gradient bg, rounded slightly less than outer
            <div
                class="w-full h-full rounded-[42px] overflow-hidden flex flex-col gap-3 px-3.5"
                style="padding-top: 42px; padding-bottom: 14px; background: linear-gradient(180deg, var(--color-bg-gradient-top), var(--color-bg-gradient-bottom));"
            >
                {children()}
            </div>

            // Dynamic Island — black pill at top
            <div
                class="absolute top-3.5 left-1/2 w-[104px] h-[30px] rounded-[15px] bg-black"
                style="transform: translateX(-50%);"
            ></div>
        </div>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Faux app screens — used inside PhoneFrame and as feature mocks
// ════════════════════════════════════════════════════════════════════════

#[component]
fn LibraryMock() -> impl IntoView {
    view! {
        <>
            <div class="flex items-center justify-between">
                <h3 class="text-2xl font-bold text-primary font-heading">"Library"</h3>
                <span class="text-xs font-semibold text-accent-text">"+ Add"</span>
            </div>
            <div class="inline-flex items-center gap-1 p-1 rounded-full bg-surface-input self-start">
                <span class="px-3 py-1 rounded-full bg-accent text-white text-[11px] font-semibold">"Pieces"</span>
                <span class="px-3 py-1 text-muted text-[11px] font-semibold">"Exercises"</span>
            </div>
            <FauxLibraryRow title="Clair de Lune".to_string() subtitle="Debussy".to_string() variant="piece".to_string() />
            <FauxLibraryRow title="Nocturne Op.9 No.2".to_string() subtitle="Chopin".to_string() variant="piece".to_string() />
            <FauxLibraryRow title="Hanon No. 1".to_string() subtitle="Hanon".to_string() variant="exercise".to_string() />
        </>
    }
}

#[component]
fn FauxLibraryRow(title: String, subtitle: String, variant: String) -> impl IntoView {
    let row_class = if variant == "exercise" {
        "accent-row accent-row--blue"
    } else {
        "accent-row"
    };

    let indicator_class = if variant == "exercise" {
        "inline-type-indicator inline-type-indicator--exercise"
    } else {
        "inline-type-indicator inline-type-indicator--piece"
    };

    let indicator_label = if variant == "exercise" {
        "Exercise"
    } else {
        "Piece"
    };

    view! {
        <div class=row_class>
            <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                <span class="text-sm font-semibold text-primary truncate">{title}</span>
                <span class="text-xs text-muted truncate">{subtitle}</span>
            </div>
            <span class=indicator_class>
                <span class="inline-type-indicator-dot"></span>
                {indicator_label}
            </span>
        </div>
    }
}

#[component]
fn PracticeMock() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center w-full h-full gap-4">
            <p class="text-[10px] font-bold tracking-[0.14em] text-faint">"NOW PRACTICING"</p>
            <h3 class="text-2xl font-bold text-primary font-heading">"Clair de Lune"</h3>
            <p class="text-xs text-muted">"Debussy"</p>
            <p
                class="text-6xl font-bold text-accent-text mt-4 mb-2"
                style="letter-spacing: -0.04em;"
            >
                "04:32"
            </p>
            <div class="w-full h-1.5 rounded-full bg-surface-input overflow-hidden">
                <div class="h-full bg-accent rounded-full" style="width: 65%;"></div>
            </div>
            <div class="flex items-center gap-2 mt-4">
                <span class="px-4 py-2 rounded-lg bg-surface-input text-xs font-semibold text-primary">"Pause"</span>
                <span class="px-4 py-2 rounded-lg bg-accent text-xs font-semibold text-white">"Next →"</span>
            </div>
        </div>
    }
}

#[component]
fn AnalyticsMock() -> impl IntoView {
    view! {
        <>
            <h3 class="text-2xl font-bold text-primary font-heading">"Analytics"</h3>
            <div class="grid grid-cols-2 gap-2">
                <FauxStatCard label="TOTAL TIME".to_string() value="12h".to_string() />
                <FauxStatCard label="SESSIONS".to_string() value="18".to_string() />
            </div>
            <FauxChart />
        </>
    }
}

#[component]
fn FauxStatCard(label: String, value: String) -> impl IntoView {
    view! {
        <div class="card p-3 flex flex-col items-start gap-1">
            <p class="text-[9px] font-bold tracking-[0.08em] text-faint">{label}</p>
            <p class="text-2xl font-bold text-primary font-heading">{value}</p>
            <p class="text-[10px] text-muted">"this month"</p>
        </div>
    }
}

#[component]
fn FauxChart() -> impl IntoView {
    let bars = [42_u16, 78, 60, 108, 84, 120, 66];
    let highlights = [false, false, false, true, false, true, false];
    let days = ["M", "T", "W", "T", "F", "S", "S"];

    view! {
        <div class="card p-3 flex flex-col gap-2">
            <p class="text-xs font-semibold text-primary">"Practice Time"</p>
            <div class="flex items-end gap-1.5 h-24">
                {bars.iter().zip(highlights.iter()).map(|(h, hi)| {
                    let bg = if *hi { "bg-accent" } else { "bg-accent/50" };
                    let style = format!("height: {}%;", (*h as f32 / 120.0 * 100.0) as u32);
                    view! {
                        <div class=format!("flex-1 rounded {bg}") style=style></div>
                    }
                }).collect_view()}
            </div>
            <div class="flex items-center gap-1.5">
                {days.iter().map(|d| view! {
                    <span class="flex-1 text-[8px] text-faint text-center">{*d}</span>
                }).collect_view()}
            </div>
        </div>
    }
}
