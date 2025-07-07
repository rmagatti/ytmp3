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

    let (auth_session, set_auth_session) = use_auth_session();

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
    let on_logout = move |_| {
        leptos::task::spawn_local(async move {
            match sign_out(set_auth_session).await {
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
        <div class="min-h-screen bg-gradient-to-br from-primary to-secondary">
            // Auth header
            <div class="navbar bg-base-300/50 backdrop-blur-sm">
                <div class="navbar-start">
                    <div class="text-base-content">
                        <span class="text-sm opacity-70">
                            "Logged in as "
                            <span class="font-medium">
                                {move || {
                                    auth_session.get()
                                        .map_or(
                                            "Guest".to_string(),
                                            |session| { session.email.clone() },
                                        )
                                }}
                            </span>
                        </span>
                    </div>
                </div>
                <div class="navbar-end">
                    <button
                        on:click=on_logout
                        class="btn btn-error btn-sm"
                    >
                        "Logout"
                    </button>
                </div>
            </div>

            // Main content
            <div
                class="flex items-center justify-center p-4"
                style="min-height: calc(100vh - 72px);"
            >
                <div class="w-full max-w-4xl">
                    <div class="card bg-base-200 shadow-2xl">
                        <div class="card-body text-center">
                            <h1 class="card-title text-5xl font-bold justify-center mb-4">
                                "YouTube to MP3 Converter"
                            </h1>
                            <p class="text-base-content/70 text-lg mb-12">
                                "Convert YouTube videos to MP3 files quickly and easily"
                            </p>

                            <div class="space-y-6">
                                <div class="flex flex-col sm:flex-row gap-4 max-w-2xl mx-auto">
                                    <input
                                        type="url"
                                        placeholder="Paste YouTube URL..."
                                        prop:value=move || url_input.get()
                                        on:input=move |ev| {
                                            url_input.set(event_target_value(&ev));
                                            error_message.set(None);
                                        }
                                        class="input input-bordered flex-1 text-center"
                                        class:input-disabled=move || is_converting.get()
                                        disabled=move || is_converting.get()
                                    />
                                    <button
                                        on:click=on_convert
                                        disabled=move || {
                                            is_converting.get() || url_input.get().is_empty()
                                        }
                                        class="btn btn-primary"
                                        class:btn-disabled=move || {
                                            is_converting.get() || url_input.get().is_empty()
                                        }
                                        class:loading=move || is_converting.get()
                                    >
                                        {move || {
                                            if is_converting.get() {
                                                "Converting..."
                                            } else {
                                                "Convert"
                                            }
                                        }}
                                    </button>
                                </div>

                                // Error message
                                {move || {
                                    error_message
                                        .get()
                                        .map(|msg| {
                                            view! {
                                                <div class="alert alert-error max-w-2xl mx-auto">
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                    </svg>
                                                    {msg}
                                                </div>
                                            }
                                        })
                                }}

                                // Progress indicator
                                {move || {
                                    is_converting
                                        .get()
                                        .then(|| {
                                            view! {
                                                <div class="text-center space-y-4">
                                                    <progress class="progress progress-primary w-full"></progress>
                                                    <p class="text-base-content/60">"Processing your video..."</p>
                                                </div>
                                            }
                                        })
                                }}

                                // Download button
                                {move || {
                                    download_url
                                        .get()
                                        .map(|url| {
                                            view! {
                                                <div class="card bg-success text-success-content max-w-md mx-auto">
                                                    <div class="card-body text-center">
                                                        <div class="flex justify-center mb-4">
                                                            <div class="avatar placeholder">
                                                                <div class="bg-success-content text-success rounded-full w-16">
                                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                                    </svg>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <h3 class="card-title justify-center text-2xl font-bold mb-4">
                                                            "Conversion Complete!"
                                                        </h3>
                                                        <div class="card-actions justify-center">
                                                            <a
                                                                href=url
                                                                download
                                                                class="btn btn-success-content"
                                                            >
                                                                "Download MP3"
                                                            </a>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        })
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
