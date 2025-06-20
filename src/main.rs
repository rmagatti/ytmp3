
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use ytmp3::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    use axum::routing::get;
    use axum::extract::Path;
    use axum::response::IntoResponse;
    
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .route("/api/download/{id}", get(download_handler))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    async fn download_handler(Path(id): Path<String>) -> impl IntoResponse {
        match ytmp3::converter::server::get_mp3_file(&id).await {
            Ok(file_content) => {
                let filename = format!("attachment; filename=\"{}.mp3\"", id);
                (
                    axum::http::StatusCode::OK,
                    [
                        ("content-type", "audio/mpeg"),
                        ("content-disposition", filename.as_str()),
                    ],
                    file_content,
                ).into_response()
            }
            Err(e) => {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    [("content-type", "text/plain")],
                    format!("Error: {}", e),
                ).into_response()
            }
        }
    }

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
