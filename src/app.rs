use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "hydrate")]
use gloo_timers::future::sleep;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConvertResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

#[server(ConvertVideo, "/api")]
pub async fn convert_video(url: String) -> Result<ConvertResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::converter::server::*;
        
        if url.is_empty() || !is_valid_youtube_url(&url) {
            return Ok(ConvertResponse {
                id: String::new(),
                status: "error".to_string(),
                message: "Please enter a valid YouTube URL".to_string(),
            });
        }
        
        match start_conversion(url).await {
            Ok(job_id) => Ok(ConvertResponse {
                id: job_id,
                status: "processing".to_string(),
                message: "Conversion started".to_string(),
            }),
            Err(e) => Ok(ConvertResponse {
                id: String::new(),
                status: "error".to_string(),
                message: format!("Failed to start conversion: {e}"),
            }),
        }
    }
    
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server function not available on client"))
    }
}

#[server(CheckStatus, "/api")]
pub async fn check_status(job_id: String) -> Result<ConvertResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::converter::server::*;
        
        match get_job_status(&job_id).await {
            Ok(status) => Ok(status),
            Err(e) => Ok(ConvertResponse {
                id: job_id,
                status: "error".to_string(),
                message: format!("Failed to check status: {e}"),
            }),
        }
    }
    
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Server function not available on client"))
    }
}

async fn poll_conversion_status(
    job_id: String,
    is_converting: RwSignal<bool>,
    download_url: RwSignal<Option<String>>,
    error_message: RwSignal<Option<String>>,
) {
    loop {
        #[cfg(feature = "hydrate")]
        sleep(std::time::Duration::from_secs(2)).await;
        
        match check_status(job_id.clone()).await {
            Ok(response) => {
                match response.status.as_str() {
                    "completed" => {
                        is_converting.set(false);
                        download_url.set(Some(format!("/api/download/{job_id}")));
                        break;
                    }
                    "error" => {
                        is_converting.set(false);
                        error_message.set(Some(response.message));
                        break;
                    }
                    _ => {
                        // Still processing, continue polling
                    }
                }
            }
            Err(e) => {
                is_converting.set(false);
                error_message.set(Some(format!("Failed to check status: {e}")));
                break;
            }
        }
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/ytmp3.css"/>

        // sets the document title
        <Title text="YouTube to MP3 Converter"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let url_input = RwSignal::new(String::new());
    let is_converting = RwSignal::new(false);
    let download_url = RwSignal::new(Option::<String>::None);
    let error_message = RwSignal::new(Option::<String>::None);
    let conversion_id = RwSignal::new(Option::<String>::None);

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
        
        spawn_local(async move {
            match convert_video(url).await {
                Ok(response) => {
                    if response.status == "processing" {
                        conversion_id.set(Some(response.id.clone()));
                        // Start polling for status
                        poll_conversion_status(response.id, is_converting, download_url, error_message).await;
                    } else {
                        is_converting.set(false);
                        error_message.set(Some(response.message));
                    }
                }
                Err(e) => {
                    is_converting.set(false);
                    error_message.set(Some(format!("Conversion failed: {e}")));
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-teal-400 via-blue-500 to-purple-600 flex items-center justify-center p-4">
            <div class="w-full max-w-4xl">
                <div class="card bg-gray-800 shadow-2xl rounded-3xl overflow-hidden">
                    <div class="card-body p-12 text-center">
                        <h1 class="text-5xl font-bold text-white mb-4">"YouTube to MP3 Converter"</h1>
                        <p class="text-gray-300 text-lg mb-12">"Convert YouTube videos to MP3 files quickly and easily"</p>
                        
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
                                    disabled=move || is_converting.get() || url_input.get().is_empty()
                                    class="btn bg-lime-400 hover:bg-lime-500 text-gray-900 border-0 rounded-full px-8 py-4 font-bold transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                                >
                                    {move || {
                                        if is_converting.get() { 
                                            view! {
                                                <span class="loading loading-spinner loading-sm mr-2"></span>
                                                "Converting..."
                                            }.into_any()
                                        } else { 
                                            view! { 
                                                "Convert Now"
                                            }.into_any()
                                        }
                                    }}
                                </button>
                            </div>
                            
                            {move || error_message.get().map(|msg| view! {
                                <div class="alert bg-red-500/20 border border-red-500/30 text-red-200 rounded-2xl">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-5 w-5" fill="none" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                    <span>{msg}</span>
                                </div>
                            })}
                            
                            {move || if is_converting.get() {
                                Some(view! {
                                    <div class="text-center space-y-4">
                                        <div class="w-full bg-gray-700 rounded-full h-2 overflow-hidden">
                                            <div class="bg-gradient-to-r from-lime-400 to-yellow-400 h-full rounded-full animate-pulse"></div>
                                        </div>
                                        <p class="text-gray-300">"Processing your video..."</p>
                                    </div>
                                })
                            } else {
                                None
                            }}
                            
                            {move || download_url.get().map(|url| view! {
                                <div class="bg-gradient-to-r from-green-500 to-emerald-500 rounded-2xl p-8 shadow-lg">
                                    <div class="flex justify-center mb-4">
                                        <div class="bg-white/20 rounded-full p-3">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                            </svg>
                                        </div>
                                    </div>
                                    <h3 class="text-2xl font-bold text-white mb-6">"Conversion Complete!"</h3>
                                    <a href=url download class="btn bg-white text-green-600 hover:bg-gray-100 border-0 rounded-full px-8 py-3 font-bold shadow-lg transition-all duration-200">"Download MP3"</a>
                                </div>
                            })}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
