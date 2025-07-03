use leptos::prelude::*;
use leptos_router::components::Redirect;

use crate::components::auth::use_auth_session;

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let (auth_session, _) = use_auth_session();

    if auth_session().is_none() {
        view! { <Redirect path="/login" /> }.into_any()
    } else {
        view! { {children()} }.into_any()
    }
}
