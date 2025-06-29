#[cfg(feature = "ssr")]
pub mod server {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::process::Stdio;
    use tempfile::TempDir;
    use tokio::process::Command;
    use uuid::Uuid;

    use crate::server::video_conversion::convert_video::ConvertResponse;

    #[derive(Debug)]
    pub struct ConversionJob {
        pub id: String,
        pub temp_dir: TempDir,
        pub mp3_path: Option<PathBuf>,
        pub status: String,
        pub error: Option<String>,
    }

    // In a real application, you'd use a proper database or redis
    // For now, we'll use a simple in-memory store
    type JobStore = std::sync::Arc<tokio::sync::RwLock<HashMap<String, ConversionJob>>>;

    static JOB_STORE: std::sync::LazyLock<JobStore> =
        std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())));

    /// Starts a new conversion job for a YouTube URL.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to create temporary directory
    /// - Failed to store job in the job store
    pub async fn start_conversion(
        url: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let job_id = Uuid::new_v4().to_string();

        // Create temporary directory for this job
        let temp_dir = tempfile::tempdir()?;

        let job = ConversionJob {
            id: job_id.clone(),
            temp_dir,
            mp3_path: None,
            status: "processing".to_string(),
            error: None,
        };

        // Store the job
        JOB_STORE.write().await.insert(job_id.clone(), job);

        // Start the conversion process in the background
        let job_id_clone = job_id.clone();
        let url_clone = url.clone();

        tokio::spawn(async move {
            process_conversion(job_id_clone, url_clone).await;
        });

        Ok(job_id)
    }

    /// Gets the current status of a conversion job.
    ///
    /// # Errors
    ///
    /// This function doesn't typically return errors, but wraps responses in Result
    /// for consistency with the API interface.
    pub async fn get_job_status(
        job_id: &str,
    ) -> Result<ConvertResponse, Box<dyn std::error::Error + Send + Sync>> {
        let jobs = JOB_STORE.read().await;

        match jobs.get(job_id) {
            Some(job) => {
                let message = if let Some(ref error) = job.error {
                    error.clone()
                } else if job.status == "completed" {
                    "Conversion completed successfully".to_string()
                } else {
                    "Processing your video...".to_string()
                };

                Ok(ConvertResponse {
                    id: job.id.clone(),
                    status: job.status.clone(),
                    message,
                })
            }
            None => Ok(ConvertResponse {
                id: job_id.to_string(),
                status: "not_found".to_string(),
                message: "Job not found".to_string(),
            }),
        }
    }

    /// Retrieves the MP3 file contents for a completed conversion job.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Job not found
    /// - Conversion not completed yet
    /// - MP3 file not found or unable to read file
    /// - File system I/O errors
    pub async fn get_mp3_file(
        job_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let jobs = JOB_STORE.read().await;

        match jobs.get(job_id) {
            Some(job) if job.status == "completed" => {
                if let Some(ref mp3_path) = job.mp3_path {
                    let file_content = tokio::fs::read(mp3_path).await?;
                    Ok(file_content)
                } else {
                    Err("MP3 file not found".into())
                }
            }
            Some(_) => Err("Conversion not completed yet".into()),
            None => Err("Job not found".into()),
        }
    }

    async fn process_conversion(job_id: String, url: String) {
        // Download video using yt-dlp
        let temp_dir_path = {
            let jobs = JOB_STORE.read().await;
            if let Some(job) = jobs.get(&job_id) {
                job.temp_dir.path().to_path_buf()
            } else {
                return;
            }
        };

        // Use yt-dlp to directly extract audio as MP3
        let output_template = temp_dir_path.join("%(title)s.%(ext)s");

        let download_result = Command::new("yt-dlp")
            .arg(&url)
            .arg("-x") // Extract audio
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg("192K")
            .arg("-o")
            .arg(&output_template)
            .arg("--restrict-filenames") // Use safe filenames
            // Enhanced anti-bot measures
            .arg("--user-agent")
            .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .arg("--referer")
            .arg("https://www.youtube.com/")
            .arg("--sleep-interval")
            .arg("2")
            .arg("--max-sleep-interval")
            .arg("8")
            .arg("--sleep-subtitles")
            .arg("2")
            .arg("--extractor-args")
            .arg("youtube:player_client=android,ios,web;include_live_dash=false")
            .arg("--no-check-certificates")
            .arg("--prefer-free-formats")
            // Additional anti-detection headers
            .arg("--add-headers")
            .arg("Accept:text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .arg("--add-headers") 
            .arg("Accept-Language:en-US,en;q=0.5")
            .arg("--add-headers")
            .arg("Accept-Encoding:gzip, deflate, br")
            .arg("--add-headers")
            .arg("DNT:1")
            .arg("--add-headers")
            .arg("Connection:keep-alive")
            .arg("--add-headers")
            .arg("Upgrade-Insecure-Requests:1")
            // Retry mechanism
            .arg("--retries")
            .arg("3")
            .arg("--retry-sleep")
            .arg("5")
            // Use different extraction methods as fallback
            .arg("--compat-options")
            .arg("prefer-legacy-http-handler")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        let mut final_mp3_path = None;

        match download_result {
            Ok(output) => {
                if output.status.success() {
                    // Find the generated MP3 file
                    if let Ok(mut entries) = tokio::fs::read_dir(&temp_dir_path).await {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let path = entry.path();
                            if path.extension().and_then(|s| s.to_str()) == Some("mp3") {
                                final_mp3_path = Some(path);
                                break;
                            }
                        }
                    }

                    // Update job status
                    let mut jobs = JOB_STORE.write().await;
                    if let Some(job) = jobs.get_mut(&job_id) {
                        if let Some(mp3_path) = final_mp3_path {
                            job.status = "completed".to_string();
                            job.mp3_path = Some(mp3_path);
                        } else {
                            job.status = "error".to_string();
                            job.error = Some("MP3 file not found after conversion".to_string());
                        }
                    }
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    let mut jobs = JOB_STORE.write().await;
                    if let Some(job) = jobs.get_mut(&job_id) {
                        job.status = "error".to_string();

                        // Provide more user-friendly error messages
                        let user_friendly_error = if error_msg
                            .contains("Sign in to confirm you're not a bot")
                        {
                            "YouTube has detected automated access and is requesting verification. This is a temporary restriction. Please try again in a few minutes, or try a different video. Some videos may be more restricted than others.".to_string()
                        } else if error_msg.contains("Video unavailable")
                            || error_msg.contains("Private video")
                        {
                            "This video is unavailable. It may be private, deleted, or restricted in your region.".to_string()
                        } else if error_msg.contains("age-restricted")
                            || error_msg.contains("age_restricted")
                        {
                            "This video is age-restricted and cannot be downloaded without authentication.".to_string()
                        } else if error_msg.contains("rate limit")
                            || error_msg.contains("too many requests")
                        {
                            "YouTube is rate limiting requests. Please wait a few minutes before trying again.".to_string()
                        } else if error_msg.contains("HTTP Error 429") {
                            "Too many requests. Please wait a few minutes before trying again."
                                .to_string()
                        } else if error_msg.contains("premieres in") {
                            "This video is a premiere that hasn't started yet. Please wait until it's available.".to_string()
                        } else if error_msg.contains("live stream") {
                            "Live streams cannot be downloaded. Please wait until the stream ends or try a regular video.".to_string()
                        } else {
                            format!(
                                "Failed to download video: {}",
                                error_msg.lines().take(2).collect::<Vec<_>>().join(" ")
                            )
                        };

                        job.error = Some(user_friendly_error);
                    }
                }
            }
            Err(e) => {
                let mut jobs = JOB_STORE.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.status = "error".to_string();
                    job.error = Some(format!("Failed to execute yt-dlp: {e}"));
                }
            }
        }
    }

    pub fn is_valid_youtube_url(url: &str) -> bool {
        url.contains("youtube.com/watch")
            || url.contains("youtu.be/")
            || url.contains("youtube.com/shorts/")
            || url.contains("m.youtube.com/watch")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Helper to clean up job store between tests
        async fn reset_job_store() {
            JOB_STORE.write().await.clear();
        }

        #[tokio::test]
        async fn test_is_valid_youtube_url() {
            assert!(is_valid_youtube_url(
                "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
            ));
            assert!(is_valid_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
            assert!(is_valid_youtube_url(
                "https://m.youtube.com/watch?v=dQw4w9WgXcQ"
            ));
            assert!(is_valid_youtube_url(
                "https://www.youtube.com/shorts/abcdef123"
            ));
            assert!(is_valid_youtube_url(
                "https://www.youtube.com/watch?v=some_id&list=PL..."
            ));
            assert!(!is_valid_youtube_url("https://www.google.com"));
            assert!(!is_valid_youtube_url(
                "https://www.youtube.com/feed/subscriptions"
            ));
            assert!(!is_valid_youtube_url(
                "https://example.com/watch?v=dQw4w9WgXcQ"
            ));
        }

        #[tokio::test]
        async fn test_start_conversion_creates_job() {
            reset_job_store().await;
            // We are not testing the conversion itself, just that the job is created.
            // The conversion process is spawned in the background and would require `yt-dlp` to be installed.
            let url = "https://www.youtube.com/watch?v=video_id".to_string();
            let job_id = start_conversion(url).await.unwrap();

            let jobs = JOB_STORE.read().await;
            let job = jobs.get(&job_id).expect("Job should be in the store");

            assert_eq!(job.id, job_id);
            assert_eq!(job.status, "processing");
            assert!(job.error.is_none());
            assert!(job.mp3_path.is_none());

            // The background task will run and likely fail because yt-dlp isn't real for the test.
            // We can't easily check the final state without more complex test setup (e.g. mocking Command).
        }

        #[tokio::test]
        async fn test_get_job_status_not_found() {
            reset_job_store().await;
            let job_id = "non-existent-job-id";
            let response = get_job_status(job_id).await.unwrap();
            assert_eq!(response.id, job_id);
            assert_eq!(response.status, "not_found");
            assert_eq!(response.message, "Job not found");
        }

        #[tokio::test]
        async fn test_get_job_status_processing() {
            reset_job_store().await;
            let url = "https://www.youtube.com/watch?v=video_id".to_string();
            let job_id = start_conversion(url).await.unwrap();

            let response = get_job_status(&job_id).await.unwrap();
            assert_eq!(response.id, job_id);
            assert_eq!(response.status, "processing");
            assert_eq!(response.message, "Processing your video...");
        }

        #[tokio::test]
        async fn test_get_mp3_file_job_not_found() {
            reset_job_store().await;
            let job_id = "non-existent-job-id";
            let result = get_mp3_file(job_id).await;
            assert!(result.is_err());
            assert_eq!(result.err().unwrap().to_string(), "Job not found");
        }

        #[tokio::test]
        async fn test_get_mp3_file_conversion_not_completed() {
            reset_job_store().await;
            let url = "https://www.youtube.com/watch?v=video_id".to_string();
            let job_id = start_conversion(url).await.unwrap();

            let result = get_mp3_file(&job_id).await;
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Conversion not completed yet"
            );
        }

        #[tokio::test]
        async fn test_get_job_status_completed() {
            reset_job_store().await;
            let job_id = "completed-job".to_string();
            let temp_dir = tempfile::tempdir().unwrap();
            let mp3_path = temp_dir.path().join("test.mp3");
            tokio::fs::write(&mp3_path, "mp3 content").await.unwrap();

            let job = ConversionJob {
                id: job_id.clone(),
                temp_dir,
                mp3_path: Some(mp3_path),
                status: "completed".to_string(),
                error: None,
            };
            JOB_STORE.write().await.insert(job_id.clone(), job);

            let response = get_job_status(&job_id).await.unwrap();
            assert_eq!(response.id, job_id);
            assert_eq!(response.status, "completed");
            assert_eq!(response.message, "Conversion completed successfully");
        }

        #[tokio::test]
        async fn test_get_mp3_file_success() {
            reset_job_store().await;
            let job_id = "completed-job-for-mp3".to_string();
            let temp_dir = tempfile::tempdir().unwrap();
            let mp3_path = temp_dir.path().join("test.mp3");
            let file_contents = b"mp3 file data";
            tokio::fs::write(&mp3_path, file_contents).await.unwrap();

            let job = ConversionJob {
                id: job_id.clone(),
                temp_dir,
                mp3_path: Some(mp3_path),
                status: "completed".to_string(),
                error: None,
            };
            JOB_STORE.write().await.insert(job_id.clone(), job);

            let result = get_mp3_file(&job_id).await.unwrap();
            assert_eq!(result, file_contents);
        }

        #[tokio::test]
        async fn test_get_job_status_error() {
            reset_job_store().await;
            let job_id = "error-job".to_string();
            let temp_dir = tempfile::tempdir().unwrap();
            let error_message = "Something went wrong".to_string();

            let job = ConversionJob {
                id: job_id.clone(),
                temp_dir,
                mp3_path: None,
                status: "error".to_string(),
                error: Some(error_message.clone()),
            };
            JOB_STORE.write().await.insert(job_id.clone(), job);

            let response = get_job_status(&job_id).await.unwrap();
            assert_eq!(response.id, job_id);
            assert_eq!(response.status, "error");
            assert_eq!(response.message, error_message);
        }
    }
}
