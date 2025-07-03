use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_navigate;

#[cfg(feature = "hydrate")]
use crate::components::auth::{sign_in_with_email, sign_up_with_email, use_auth_session};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);
    let (is_loading, set_is_loading) = signal(false);

    #[cfg(feature = "hydrate")]
    let (_, set_auth_session) = use_auth_session();

    #[cfg(feature = "hydrate")]
    let navigate = use_navigate();
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let email_val = email.get();
        let password_val = password.get();

        if email_val.is_empty() || password_val.is_empty() {
            set_error_message.set(Some("Please fill in all fields".to_string()));
            return;
        }

        // Clone values for the async block
        #[cfg(feature = "hydrate")]
        let navigate = navigate.clone();

        spawn_local(async move {
            set_is_loading.set(true);
            set_error_message.set(None);
            set_success_message.set(None);

            #[cfg(feature = "hydrate")]
            {
                // Use spawn_local for WASM compatibility
                use leptos::logging;

                match sign_in_with_email(email_val.clone(), password_val.clone(), set_auth_session).await {
                    Ok(auth) => {
                        if auth.data.session.is_some() {
                            logging::log!("Login successful, session found");
                            navigate("/", Default::default());
                        } else {
                            logging::log!(
                                "Login successful, but no session. User needs to confirm email."
                            );
                            set_success_message.set(Some(
                                "Please check your email to confirm your account. You can view test emails at http://localhost:54324"
                                    .to_string(),
                            ));
                        }
                    }
                    Err(login_error) => {
                        logging::log!("Login failed, attempting to sign up: {:?}", login_error);

                        // If login failed (user might not exist), try to sign up
                        match sign_up_with_email(email_val, password_val, set_auth_session).await {
                            Ok(auth) => {
                                logging::log!("Signup response: {:?}", auth);

                                if auth.data.session.is_some() {
                                    logging::log!("Signup successful, session found");
                                    navigate("/", Default::default());
                                } else {
                                    logging::log!("Signup successful, but no session. User needs to confirm email.");
                                    set_success_message.set(Some(
                                        "Account created! Please check your email to confirm your account."
                                            .to_string(),
                                    ));
                                }
                            }
                            Err(signup_error) => {
                                logging::error!("Both login and signup failed: {:?}", signup_error);
                                set_error_message.set(Some(
                                    "Unable to sign in or create account. Please check your credentials."
                                        .to_string(),
                                ));
                            }
                        }
                    }
                }
            }

            set_is_loading.set(false);
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-teal-400 via-blue-500 to-purple-600 flex items-center justify-center p-4">
            <div class="w-full max-w-md">
                <div class="card bg-gray-800 shadow-2xl rounded-3xl overflow-hidden">
                    <div class="card-body p-8 text-center">
                        <h1 class="text-4xl font-bold text-white mb-4">"Welcome"</h1>
                        <p class="text-gray-300 text-lg mb-8">
                            "Enter your credentials to continue"
                        </p>

                        {move || {
                            error_message
                                .get()
                                .map(|msg| {
                                    view! {
                                        <div class="bg-red-500/20 border border-red-500 text-red-200 px-4 py-3 rounded-lg mb-6">
                                            {msg}
                                        </div>
                                    }
                                })
                        }}

                        {move || {
                            success_message
                                .get()
                                .map(|msg| {
                                    view! {
                                        <div class="bg-green-500/20 border border-green-500 text-green-200 px-4 py-3 rounded-lg mb-6">
                                            {msg}
                                        </div>
                                    }
                                })
                        }}

                        <form class="space-y-6" on:submit=on_submit>
                            <div class="space-y-2">
                                <label
                                    class="block text-left text-gray-300 font-medium"
                                    for="email"
                                >
                                    "Email"
                                </label>
                                <input
                                    type="email"
                                    id="email"
                                    name="email"
                                    class="w-full bg-white border-0 rounded-full px-6 py-4 text-gray-800 placeholder-gray-500 focus:outline-none focus:ring-4 focus:ring-lime-400/50"
                                    placeholder="Enter your email"
                                    prop:value=move || email.get()
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    required
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            <div class="space-y-2">
                                <label
                                    class="block text-left text-gray-300 font-medium"
                                    for="password"
                                >
                                    "Password"
                                </label>
                                <input
                                    type="password"
                                    id="password"
                                    name="password"
                                    class="w-full bg-white border-0 rounded-full px-6 py-4 text-gray-800 placeholder-gray-500 focus:outline-none focus:ring-4 focus:ring-lime-400/50"
                                    placeholder="Enter your password"
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    required
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            <div class="pt-4">
                                <button
                                    type="submit"
                                    class="w-full bg-lime-400 hover:bg-lime-500 text-gray-900 border-0 rounded-full px-8 py-4 font-bold transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || is_loading.get()
                                >
                                    {move || {
                                        if is_loading.get() {
                                            "Processing..."
                                        } else {
                                            "Login / Sign Up"
                                        }
                                    }}
                                </button>
                            </div>
                        </form>

                        <div class="flex items-center my-8">
                            <div class="flex-1 h-px bg-gray-600"></div>
                            <span class="px-4 text-gray-400 text-sm">"OR"</span>
                            <div class="flex-1 h-px bg-gray-600"></div>
                        </div>

                        <div class="space-y-3">
                            <p class="text-gray-300 text-sm">
                                "Forgot your password? "
                                <a
                                    href="/reset-password"
                                    class="text-purple-400 hover:text-purple-300 font-medium transition-colors"
                                >
                                    "Reset it here"
                                </a>
                            </p>
                        </div>
                    </div>
                </div>

                <div class="text-center mt-4">
                    <a href="/" class="text-white/70 hover:text-white text-sm transition-colors">
                        "← Back to Home"
                    </a>
                </div>
            </div>
        </div>
    }
}
