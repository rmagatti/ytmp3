use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_navigate;

#[cfg(feature = "hydrate")]
use crate::components::auth::sign_out;

/// Logout page component that automatically signs out the user and redirects
#[component]
pub fn LogoutPage() -> impl IntoView {
    let (is_loading, set_is_loading) = signal(true);
    let (error_message, _set_error_message) = signal(None::<String>);

    #[cfg(feature = "hydrate")]
    let navigate = use_navigate();

    // Automatically sign out when component mounts
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        let navigate = navigate.clone();
        spawn_local(async move {
            #[cfg(feature = "hydrate")]
            {
                use leptos::logging;

                match sign_out().await {
                    Ok(res) => {
                        logging::log!("Successfully signed out {res:?}");
                        navigate("/login", Default::default());
                    }
                    Err(e) => {
                        logging::error!("Failed to sign out: {:?}", e);
                        _set_error_message
                            .set(Some("Failed to sign out. Please try again.".to_string()));
                        set_is_loading.set(false);
                    }
                }
            }

            #[cfg(not(feature = "hydrate"))]
            {
                // On server side, just redirect to home
                set_is_loading.set(false);
            }
        });
    });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-teal-400 via-blue-500 to-purple-600 flex items-center justify-center p-4">
            <div class="w-full max-w-md">
                <div class="card bg-gray-800 shadow-2xl rounded-3xl overflow-hidden">
                    <div class="card-body p-8 text-center">
                        {move || {
                            if let Some(error) = error_message.get() {
                                view! {
                                    <div>
                                        <h1 class="text-4xl font-bold text-white mb-4">"Sign Out Error"</h1>
                                        <div class="bg-red-500/20 border border-red-500 text-red-200 px-4 py-3 rounded-lg mb-6">
                                            {error}
                                        </div>
                                        <a href="/" class="btn bg-lime-400 hover:bg-lime-500 text-gray-900 border-0 rounded-full px-8 py-4 font-bold">
                                            "Go to Home"
                                        </a>
                                    </div>
                                }.into_any()
                            } else if is_loading.get() {
                                view! {
                                    <div>
                                        <h1 class="text-4xl font-bold text-white mb-4">"Signing Out..."</h1>
                                        <div class="flex justify-center mb-6">
                                            <span class="loading loading-spinner loading-lg text-lime-400"></span>
                                        </div>
                                        <p class="text-gray-300">"Please wait while we sign you out"</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div>
                                        <h1 class="text-4xl font-bold text-white mb-4">"Signed Out"</h1>
                                        <p class="text-gray-300 mb-6">"You have been successfully signed out"</p>
                                        <a href="/" class="btn bg-lime-400 hover:bg-lime-500 text-gray-900 border-0 rounded-full px-8 py-4 font-bold">
                                            "Go to Home"
                                        </a>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
