use app::domain::services::video_converter::server::get_mp3_file;
use axum::extract::Path;
use axum::response::IntoResponse;

pub async fn download_handler(Path(id): Path<String>) -> impl IntoResponse {
    match get_mp3_file(&id).await {
        Ok(file_content) => {
            let filename = format!("attachment; filename=\"{id}.mp3\"");
            (
                axum::http::StatusCode::OK,
                [
                    ("content-type", "audio/mpeg"),
                    ("content-disposition", filename.as_str()),
                ],
                file_content,
            )
                .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::NOT_FOUND,
            [("content-type", "text/plain")],
            format!("Error: {e}"),
        )
            .into_response(),
    }
}
