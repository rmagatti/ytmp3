use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use gloo_timers::future::sleep;

use crate::domain::services::video_converter::ConvertResponse;

#[server(CheckStatus, "/api")]
pub async fn check_status(job_id: String) -> Result<ConvertResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::domain::services::video_converter::server::get_job_status;

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
        Err(ServerFnError::new(
            "Server function not available on client",
        ))
    }
}

pub async fn poll_conversion_status(
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
