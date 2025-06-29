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
        Err(ServerFnError::new(
            "Server function not available on client",
        ))
    }
}
