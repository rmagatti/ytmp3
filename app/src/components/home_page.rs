use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_navigate;

#[cfg(feature = "hydrate")]
use crate::auth::sign_out;

use crate::{
    auth::use_auth_session,
    domain::services::{check_status::poll_conversion_status, video_converter::convert_video},
};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    let url_input = RwSignal::new(String::new());
    let is_converting = RwSignal::new(false);
    let download_url = RwSignal::new(Option::<String>::None);
    let error_message = RwSignal::new(Option::<String>::None);
    let conversion_id = RwSignal::new(Option::<String>::None);

    let (auth_session, _set_auth_session) = use_auth_session();

    let on_convert = move |_| {
        let url = url_input.get();
        if url.is_empty() {
            error_message.set(Some("Please enter a YouTube URL".to_string()));
            return;
        }

        // Reset states
        error_message.set(None);
        download_url.set(None);
        is_converting.set(true);
        conversion_id.set(None);

        // Start conversion
        leptos::task::spawn_local(async move {
            match convert_video(url).await {
                Ok(response) => {
                    if response.status == "error" {
                        is_converting.set(false);
                        error_message.set(Some(response.message));
                    } else {
                        conversion_id.set(Some(response.id.clone()));
                        // Poll for status
                        poll_conversion_status(
                            response.id,
                            is_converting,
                            download_url,
                            error_message,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    is_converting.set(false);
                    error_message.set(Some(format!("Conversion failed: {e}")));
                }
            }
        });
    };

    #[cfg(feature = "hydrate")]
    let _navigate = use_navigate();

    #[cfg(feature = "hydrate")]
    let on_logout = move |_| {
        leptos::task::spawn_local(async move {
            match sign_out(_set_auth_session).await {
                Ok(_) => {
                    let navigate = use_navigate();
                    navigate("/login", Default::default());
                }
                Err(e) => {
                    leptos::logging::error!("Failed to sign out: {:?}", e);
                }
            }
        });
    };

    #[cfg(not(feature = "hydrate"))]
    let on_logout = move |_| {};

    view! {
        <div class="min-h-screen bg-gradient-to-br from-primary via-secondary to-accent relative">
            // Subtle background pattern
            <div class="absolute inset-0 opacity-5">
                <div class="absolute inset-0" style="background-image: radial-gradient(circle at 25% 25%, currentColor 2px, transparent 2px), radial-gradient(circle at 75% 75%, currentColor 2px, transparent 2px); background-size: 100px 100px;"></div>
            </div>

            // daisyUI navbar with enhanced styling
            <div class="navbar bg-base-300/70 backdrop-blur-md shadow-lg sticky top-0 z-50">
                <div class="navbar-start">
                    <div class="btn btn-ghost normal-case text-xl font-bold">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                        </svg>
                        "MP3 Converter"
                    </div>
                </div>
                <div class="navbar-center hidden lg:flex">
                    <div class="text-base-content/70 text-sm">
                        "Welcome back, "
                        <span class="font-semibold text-primary">
                            {move || {
                                auth_session.get()
                                    .map_or(
                                        "Guest".to_string(),
                                        |session| { session.email.split('@').next().unwrap_or("User").to_string() },
                                    )
                            }}
                        </span>
                    </div>
                </div>
                <div class="navbar-end">
                    <div class="dropdown dropdown-end">
                        <div tabindex="0" role="button" class="btn btn-ghost btn-circle avatar">
                            <div class="w-10 rounded-full bg-gradient-to-r from-primary to-secondary flex items-center justify-center">
                                <span class="text-xs text-base-100 font-bold">
                                    {move || {
                                        auth_session.get()
                                            .map_or(
                                                "G".to_string(),
                                                |session| { session.email.chars().next().unwrap_or('U').to_uppercase().to_string() },
                                            )
                                    }}
                                </span>
                            </div>
                        </div>
                        <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-[1] w-52 p-2 shadow-xl">
                            <li><button on:click=on_logout class="text-error">Logout</button></li>
                        </ul>
                    </div>
                </div>
            </div>

            // Main content using daisyUI hero
            <div class="hero min-h-screen relative">
                <div class="hero-content text-center max-w-none w-full">
                    <div class="max-w-6xl w-full space-y-12">
                        // Hero section with daisyUI components
                        <div class="space-y-8">
                            <div class="avatar">
                                <div class="w-24 mask mask-hexagon bg-gradient-to-r from-primary to-secondary p-6">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 text-base-100" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                                    </svg>
                                </div>
                            </div>
                            <h1 class="text-5xl md:text-6xl font-bold">
                                <span class="bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                                    "YouTube to MP3 Converter"
                                </span>
                            </h1>
                            <p class="text-xl text-base-content/80 max-w-2xl mx-auto">
                                "Transform your favorite YouTube videos into high-quality MP3 files quickly and easily"
                            </p>
                        </div>

                        // Main converter card using daisyUI card
                        <div class="card bg-base-200/80 backdrop-blur-md shadow-2xl border border-base-300/50">
                            <div class="card-body p-8 lg:p-12">
                                // Form using daisyUI form controls
                                <div class="form-control w-full max-w-4xl mx-auto space-y-6">
                                    <label class="label">
                                        <span class="label-text text-lg font-semibold">"YouTube Video URL"</span>
                                        <span class="label-text-alt">"Paste any YouTube link"</span>
                                    </label>
                                    
                                    <div class="join w-full">
                                        <input
                                            type="url"
                                            placeholder="https://www.youtube.com/watch?v=..."
                                            prop:value=move || url_input.get()
                                            on:input=move |ev| {
                                                url_input.set(event_target_value(&ev));
                                                error_message.set(None);
                                            }
                                            class="input input-bordered input-lg join-item flex-1"
                                            class:input-disabled=move || is_converting.get()
                                            disabled=move || is_converting.get()
                                        />
                                        <button
                                            on:click=on_convert
                                            disabled=move || {
                                                is_converting.get() || url_input.get().is_empty()
                                            }
                                            class="btn btn-primary btn-lg join-item"
                                            class:loading=move || is_converting.get()
                                        >
                                            {move || {
                                                if is_converting.get() {
                                                    "Converting..."
                                                } else {
                                                    "Convert to MP3"
                                                }
                                            }}
                                        </button>
                                    </div>
                                </div>

                                // Status messages using daisyUI alerts
                                <div class="space-y-6 mt-8">
                                    // Error alert
                                    {move || {
                                        error_message
                                            .get()
                                            .map(|msg| {
                                                view! {
                                                    <div class="alert alert-error shadow-lg max-w-2xl mx-auto">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <span>{msg}</span>
                                                    </div>
                                                }
                                            })
                                    }}

                                    // Loading state using daisyUI loading components
                                    {move || {
                                        is_converting
                                            .get()
                                            .then(|| {
                                                view! {
                                                    <div class="alert alert-info shadow-lg max-w-2xl mx-auto">
                                                        <span class="loading loading-spinner loading-md"></span>
                                                        <div class="flex flex-col items-start">
                                                            <span class="font-semibold">"Processing your video..."</span>
                                                            <span class="text-sm opacity-70">"This may take a few moments"</span>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                    }}

                                    // Success state using daisyUI success alert
                                    {move || {
                                        download_url
                                            .get()
                                            .map(|url| {
                                                view! {
                                                    <div class="alert alert-success shadow-lg max-w-md mx-auto">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <div class="flex flex-col items-start">
                                                            <span class="font-semibold">"Conversion complete!"</span>
                                                            <a
                                                                href=url
                                                                download
                                                                class="btn btn-sm btn-success mt-2"
                                                            >
                                                                "Download MP3"
                                                            </a>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                    }}
                                </div>
                            </div>

                        // Features using daisyUI stats
                        <div class="stats stats-vertical lg:stats-horizontal shadow-xl bg-base-200/60 backdrop-blur-sm">
                            <div class="stat place-items-center">
                                <div class="stat-figure text-primary">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                    </svg>
                                </div>
                                <div class="stat-title">"Speed"</div>
                                <div class="stat-value text-primary">"Fast"</div>
                                <div class="stat-desc">"Lightning quick conversion"</div>
                            </div>
                            
                            <div class="stat place-items-center">
                                <div class="stat-figure text-secondary">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
                                    </svg>
                                </div>
                                <div class="stat-title">"Quality"</div>
                                <div class="stat-value text-secondary">"192kbps"</div>
                                <div class="stat-desc">"Good quality audio"</div>
                            </div>
                            
                            <div class="stat place-items-center">
                                <div class="stat-figure text-accent">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                    </svg>
                                </div>
                                <div class="stat-title">"Security"</div>
                                <div class="stat-value text-accent">"Safe"</div>
                                <div class="stat-desc">"Privacy protected"</div>
                            </div>
                        </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
