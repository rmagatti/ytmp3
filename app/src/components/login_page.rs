#[allow(unused_imports)]
use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local, web_sys};
#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_navigate;

#[cfg(feature = "hydrate")]
use crate::auth::{get_user, sign_in_with_email, sign_up_with_email, use_auth_session};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error_message, set_error_message) = signal(None::<String>);
    let (success_message, set_success_message) = signal(None::<String>);
    let (is_loading, set_is_loading) = signal(false);

    #[cfg(feature = "hydrate")]
    let (auth_session, set_auth_session) = use_auth_session();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if auth_session.get().is_some() {
            let navigate = use_navigate();
            navigate("/", Default::default());
        }
    });

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        use leptos::logging;
        if let Some(window) = web_sys::window() {
            if let Ok(location) = window.location().hash() {
                if !location.is_empty() {
                    if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&location[1..]) {
                        if let (Some(access_token), Some(refresh_token), Some(expires_at_str)) = (
                            params.get("access_token"),
                            params.get("refresh_token"),
                            params.get("expires_at"),
                        ) {
                            if let Ok(expires_at) = expires_at_str.parse::<i64>() {
                                let at = access_token.clone();
                                spawn_local(async move {
                                    match get_user(&at).await {
                                        Ok(user) => {
                                            let session =
                                                crate::domain::entities::auth::AuthSession {
                                                    user_id: user.id,
                                                    access_token,
                                                    refresh_token,
                                                    email: user.email,
                                                    role: user.role,
                                                    expires_at,
                                                    email_confirmed_at: user.email_confirmed_at,
                                                    last_sign_in_at: user.last_sign_in_at,
                                                    is_anonymous: user.is_anonymous,
                                                };
                                            set_auth_session.set(Some(session));
                                            logging::log!(
                                                "Session restored from URL, redirecting..."
                                            );
                                            let navigate = use_navigate();
                                            navigate("/", Default::default());
                                        }
                                        Err(e) => {
                                            logging::error!(
                                                "Failed to get user from token: {:?}",
                                                e
                                            );
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }
    });

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

                match sign_in_with_email(email_val.clone(), password_val.clone(), set_auth_session)
                    .await
                {
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
        <div class="min-h-screen bg-gradient-to-br from-primary via-secondary to-accent relative">
            // Subtle background pattern
            <div class="absolute inset-0 opacity-5">
                <div class="absolute inset-0" style="background-image: radial-gradient(circle at 25% 25%, currentColor 2px, transparent 2px), radial-gradient(circle at 75% 75%, currentColor 2px, transparent 2px); background-size: 100px 100px;"></div>
            </div>

            <div class="hero min-h-screen relative">
                <div class="hero-content flex-col max-w-none w-full">
                    <div class="max-w-4xl w-full space-y-8">
                        // Hero section with enhanced branding
                        <div class="text-center space-y-6">
                            <div class="avatar">
                                <div class="w-20 mask mask-hexagon bg-gradient-to-r from-primary to-secondary p-5">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-base-100" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                    </svg>
                                </div>
                            </div>
                            <h1 class="text-4xl md:text-5xl font-bold">
                                <span class="bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent drop-shadow-[0_2px_4px_rgba(0,0,0,0.3)]">
                                    "Welcome Back"
                                </span>
                            </h1>
                            <p class="text-lg text-base-content/80 max-w-md mx-auto">
                                "Sign in to your account or create a new one to get started"
                            </p>
                        </div>

                        // Main login card with enhanced styling
                        <div class="card bg-base-200/80 backdrop-blur-md shadow-2xl border border-base-300/50 max-w-md mx-auto">
                            <div class="card-body p-8">
                                // Status messages with better styling
                                <div class="space-y-4">
                                    {move || {
                                        error_message
                                            .get()
                                            .map(|msg| {
                                                view! {
                                                    <div class="alert alert-error shadow-lg">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        <span>{msg}</span>
                                                    </div>
                                                }
                                            })
                                    }}

                                    {move || {
                                        success_message
                                            .get()
                                            .map(|msg| {
                                                view! {
                                                    <div class="alert alert-success shadow-lg">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                                        </svg>
                                                        <span>{msg}</span>
                                                    </div>
                                                }
                                            })
                                    }}
                                </div>

                                // Enhanced form with better styling
                                <form class="space-y-6 mt-6" on:submit=on_submit>
                                    <div class="form-control">
                                        <label class="label" for="email">
                                            <span class="label-text font-semibold">"Email Address"</span>
                                        </label>
                                        <div class="relative">
                                            <input
                                                type="email"
                                                id="email"
                                                name="email"
                                                class="input input-bordered input-lg w-full pl-12"
                                                placeholder="your@email.com"
                                                prop:value=move || email.get()
                                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                                required
                                                disabled=move || is_loading.get()
                                            />
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 absolute left-3 top-1/2 transform -translate-y-1/2 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207" />
                                            </svg>
                                        </div>
                                    </div>

                                    <div class="form-control">
                                        <label class="label" for="password">
                                            <span class="label-text font-semibold">"Password"</span>
                                        </label>
                                        <div class="relative">
                                            <input
                                                type="password"
                                                id="password"
                                                name="password"
                                                class="input input-bordered input-lg w-full pl-12"
                                                placeholder="Enter your password"
                                                prop:value=move || password.get()
                                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                                required
                                                disabled=move || is_loading.get()
                                            />
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 absolute left-3 top-1/2 transform -translate-y-1/2 text-base-content/50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                            </svg>
                                        </div>
                                    </div>

                                    <div class="form-control">
                                        <button
                                            type="submit"
                                            class="btn btn-primary btn-lg w-full shadow-lg hover:shadow-xl transition-all duration-200"
                                            class:loading=move || is_loading.get()
                                            disabled=move || is_loading.get()
                                        >
                                            {move || {
                                                if is_loading.get() {
                                                    "Signing you in..."
                                                } else {
                                                    "Sign In / Create Account"
                                                }
                                            }}
                                        </button>
                                    </div>
                                </form>

                                <div class="divider my-6">"OR"</div>

                                // Enhanced footer links
                                <div class="space-y-4 text-center">
                                    <p class="text-sm text-base-content/70">
                                        "Forgot your password? "
                                        <a href="/reset-password" class="link link-primary font-medium hover:link-hover">
                                            "Reset it here"
                                        </a>
                                    </p>

                                    <div class="pt-4 border-t border-base-300/50">
                                        <a href="/" class="btn btn-ghost btn-sm gap-2">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
                                            </svg>
                                            "Back to Home"
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Features showcase for login page - properly aligned
                        <div class="flex justify-center">
                            <div class="stats stats-vertical lg:stats-horizontal shadow-xl bg-base-200/60 backdrop-blur-sm">
                                <div class="stat place-items-center">
                                    <div class="stat-figure text-primary">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                        </svg>
                                    </div>
                                    <div class="stat-title text-sm">"Secure"</div>
                                    <div class="stat-desc">"Protected login"</div>
                                </div>
                                
                                <div class="stat place-items-center">
                                    <div class="stat-figure text-secondary">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                                        </svg>
                                    </div>
                                    <div class="stat-title text-sm">"Fast"</div>
                                    <div class="stat-desc">"Quick access"</div>
                                </div>
                                
                                <div class="stat place-items-center">
                                    <div class="stat-figure text-accent">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192L5.636 18.364M12 12h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                        </svg>
                                    </div>
                                    <div class="stat-title text-sm">"Easy"</div>
                                    <div class="stat-desc">"Simple signup"</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
