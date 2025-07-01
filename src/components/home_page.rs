use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_navigate;

#[cfg(feature = "hydrate")]
use crate::components::auth::{sign_out, use_auth_session};

use crate::server::video_conversion::{
    check_status::poll_conversion_status, convert_video::convert_video,
};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    let url_input = RwSignal::new(String::new());
    let is_converting = RwSignal::new(false);
    let download_url = RwSignal::new(Option::<String>::None);
    let error_message = RwSignal::new(Option::<String>::None);
    let conversion_id = RwSignal::new(Option::<String>::None);

    #[cfg(feature = "hydrate")]
    let (auth_session, _, _) = use_auth_session();
    let is_loading = RwSignal::new(true);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let auth_session = auth_session.get();
        if !auth_session.access_token.is_empty() {
            is_loading.set(false);
        }
    });
    #[cfg(not(feature = "hydrate"))]
    let auth_session = RwSignal::new(crate::domain::entities::auth::AuthSession::default());

    // Redirect to login if not authenticated
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let auth_session = auth_session.get();
        if !is_loading.get() && auth_session.access_token.is_empty() {
            leptos::logging::log!("Redirecting to login");
            let navigate = use_navigate();
            navigate("/login", Default::default());
        }
    });

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
        spawn_local(async move {
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
        spawn_local(async {
            if sign_out().await.is_ok() {
                let navigate = use_navigate();
                navigate("/login", Default::default());
            } else {
                leptos::logging::log!("Logout failed");
            }
        });
    };

    #[cfg(not(feature = "hydrate"))]
    let on_logout = move |_| {};

    view! {
        <div class="min-h-screen bg-gradient-to-br from-teal-400 via-blue-500 to-purple-600">
            // Auth header
            <div class="bg-gray-900/50 backdrop-blur-sm">
                <div class="max-w-6xl mx-auto px-4 py-4 flex justify-between items-center">
                    <div class="text-white">
                        {move || {
                            if is_loading.get() {
                                view! { <span class="text-sm text-gray-300">"Loading..."</span> }
                                    .into_view()
                            } else {
                                let session = auth_session.get();
                                if !session.email.is_empty() {
                                    view! {
                                        <span class="text-sm text-gray-300">
                                            "Logged in as "
                                            <span class="font-medium text-white">{session.email}</span>
                                        </span>
                                    }
                                        .into_view()
                                } else {
                                    view! {
                                        <span class="text-sm text-gray-300">
                                            "Logged in"
                                            <span class="font-medium text-white"></span>
                                        </span>
                                    }
                                        .into_view()
                                }
                            }
                        }} 
                    </div>
                    <button
                        on:click=on_logout
                        class="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
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
                    <div class="card bg-gray-800 shadow-2xl rounded-3xl overflow-hidden">
                        <div class="card-body p-12 text-center">
                            <h1 class="text-5xl font-bold text-white mb-4">
                                "YouTube to MP3 Converter"
                            </h1>
                            <p class="text-gray-300 text-lg mb-12">
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
                                        class="input bg-white border-0 rounded-full px-6 py-4 text-gray-800 placeholder-gray-500 flex-1 text-center focus:outline-none focus:ring-4 focus:ring-lime-400/50"
                                        class:opacity-50=move || is_converting.get()
                                        disabled=move || is_converting.get()
                                    />
                                    <button
                                        on:click=on_convert
                                        disabled=move || {
                                            is_converting.get() || url_input.get().is_empty()
                                        }
                                        class="btn rounded-full px-8 py-4 font-bold transition-all duration-200"
                                        class:bg-gradient-to-r=move || !is_converting.get()
                                        class:from-lime-400=move || !is_converting.get()
                                        class:to-yellow-400=move || !is_converting.get()
                                        class:text-gray-800=move || !is_converting.get()
                                        class:hover:shadow-lg=move || !is_converting.get()
                                        class:bg-gray-600=move || is_converting.get()
                                        class:text-gray-400=move || is_converting.get()
                                        class:cursor-not-allowed=move || {
                                            is_converting.get() || url_input.get().is_empty()
                                        }
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
                                                <div class="alert bg-red-500/20 border border-red-500/30 text-red-300 rounded-xl p-4 max-w-2xl mx-auto animate-fadeIn">
                                                    <svg
                                                        class="w-5 h-5 inline-block mr-2"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        viewBox="0 0 24 24"
                                                    >
                                                        <path
                                                            stroke-linecap="round"
                                                            stroke-linejoin="round"
                                                            stroke-width="2"
                                                            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                                                        />
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
                                                    <div class="w-full bg-gray-700 rounded-full h-2 overflow-hidden">
                                                        <div class="bg-gradient-to-r from-lime-400 to-yellow-400 h-full animate-pulse" />
                                                    </div>
                                                    <p class="text-gray-400">"Processing your video..."</p>
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
                                                <div class="bg-gradient-to-r from-green-500 to-emerald-500 rounded-2xl p-8 max-w-md mx-auto animate-fadeIn">
                                                    <div class="flex justify-center mb-4">
                                                        <div class="bg-white/20 rounded-full p-3">
                                                            <svg
                                                                class="w-8 h-8 text-white"
                                                                fill="none"
                                                                stroke="currentColor"
                                                                viewBox="0 0 24 24"
                                                            >
                                                                <path
                                                                    stroke-linecap="round"
                                                                    stroke-linejoin="round"
                                                                    stroke-width="2"
                                                                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                                                />
                                                            </svg>
                                                        </div>
                                                    </div>
                                                    <h3 class="text-white text-2xl font-bold mb-4">
                                                        "Conversion Complete!"
                                                    </h3>
                                                    <a
                                                        href=url
                                                        download
                                                        class="btn bg-white text-green-600 hover:bg-gray-100 border-0 rounded-full px-8 py-3 font-bold shadow-lg transition-all duration-200"
                                                    >
                                                        "Download MP3"
                                                    </a>
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
