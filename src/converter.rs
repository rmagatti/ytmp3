#[cfg(feature = "ssr")]
pub mod server {
    use crate::app::ConvertResponse;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::process::Stdio;
    use tempfile::TempDir;
    use tokio::process::Command;
    use uuid::Uuid;

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

    static JOB_STORE: std::sync::LazyLock<JobStore> = std::sync::LazyLock::new(|| {
        std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new()))
    });

    #[cfg(test)]
    pub(crate) fn get_job_store() -> &'static JobStore {
        &JOB_STORE
    }

    /// Starts a new conversion job for a YouTube URL.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Unable to create temporary directory
    /// - Failed to store job in the job store
    pub async fn start_conversion(url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    pub async fn get_job_status(job_id: &str) -> Result<ConvertResponse, Box<dyn std::error::Error + Send + Sync>> {
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
    pub async fn get_mp3_file(job_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
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
            .arg("--user-agent")
            .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .arg("--sleep-interval")
            .arg("1")
            .arg("--max-sleep-interval")
            .arg("5")
            .arg("--extractor-args")
            .arg("youtube:player_client=android,web")
            .arg("--no-check-certificates")
            .arg("--prefer-free-formats")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        let mut final_mp3_path = None;

        match download_result {
            Ok(output) => {
                if output.status.success() {
                    // Find the generated MP3 file
                    if let Ok(entries) = std::fs::read_dir(&temp_dir_path) {
                        for entry in entries.flatten() {
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
                        let user_friendly_error = if error_msg.contains("Sign in to confirm you're not a bot") {
                            "YouTube has detected automated access. This video may be restricted or require verification. Please try a different video or try again later.".to_string()
                        } else if error_msg.contains("Video unavailable") {
                            "This video is unavailable. It may be private, deleted, or restricted in your region.".to_string()
                        } else if error_msg.contains("age-restricted") {
                            "This video is age-restricted and cannot be downloaded without authentication.".to_string()
                        } else {
                            format!("Failed to download video: {}", error_msg.lines().next().unwrap_or("Unknown error"))
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
        url.contains("youtube.com/watch") || 
        url.contains("youtu.be/") || 
        url.contains("youtube.com/shorts/") ||
        url.contains("m.youtube.com/watch")
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::server::*;
    use std::time::Duration;
    use tempfile::NamedTempFile;
    use tokio::time::timeout;

    #[test]
    fn test_is_valid_youtube_url() {
        // Valid URLs
        assert!(is_valid_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(is_valid_youtube_url("https://youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(is_valid_youtube_url("http://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(is_valid_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
        assert!(is_valid_youtube_url("https://www.youtube.com/shorts/dQw4w9WgXcQ"));
        assert!(is_valid_youtube_url("https://m.youtube.com/watch?v=dQw4w9WgXcQ"));
        
        // Invalid URLs
        assert!(!is_valid_youtube_url("https://vimeo.com/123456"));
        assert!(!is_valid_youtube_url("https://www.google.com"));
        assert!(!is_valid_youtube_url("not a url"));
        assert!(!is_valid_youtube_url(""));
        assert!(!is_valid_youtube_url("https://youtube.com/playlist?list=123"));
        assert!(!is_valid_youtube_url("https://youtube.com"));
    }

    #[tokio::test]
    async fn test_start_conversion_creates_job() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        
        let result = start_conversion(url).await;
        assert!(result.is_ok());
        
        let job_id = result.expect("Expected job ID");
        assert!(!job_id.is_empty());
        
        // Verify job was created in store
        let status = get_job_status(&job_id).await.expect("Expected job status");
        assert_eq!(status.id, job_id);
        assert_eq!(status.status, "processing");
    }

    #[tokio::test]
    async fn test_get_job_status_not_found() {
        let result = get_job_status("non-existent-job-id").await;
        assert!(result.is_ok());
        
        let status = result.expect("Expected status response");
        assert_eq!(status.status, "not_found");
        assert_eq!(status.message, "Job not found");
    }

    #[tokio::test]
    async fn test_get_job_status_processing() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        let job_id = start_conversion(url).await.expect("Expected job ID");
        
        let status = get_job_status(&job_id).await.expect("Expected job status");
        assert_eq!(status.id, job_id);
        assert_eq!(status.status, "processing");
        assert_eq!(status.message, "Processing your video...");
    }

    #[tokio::test]
    async fn test_get_mp3_file_job_not_found() {
        let result = get_mp3_file("non-existent-job-id").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Job not found");
    }

    #[tokio::test]
    async fn test_get_mp3_file_not_completed() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        let job_id = start_conversion(url).await.expect("Expected job ID");
        
        let result = get_mp3_file(&job_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Conversion not completed yet");
    }

    #[tokio::test]
    async fn test_get_mp3_file_completed_but_no_file() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        let job_id = start_conversion(url).await.expect("Expected job ID");
        
        // Manually mark job as completed but without MP3 path
        {
            let mut jobs = get_job_store().write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = "completed".to_string();
                // mp3_path remains None
            }
        }
        
        let result = get_mp3_file(&job_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "MP3 file not found");
    }

    #[tokio::test]
    async fn test_get_mp3_file_completed_with_file() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        let job_id = start_conversion(url).await.expect("Expected job ID");
        
        // Create a temporary MP3 file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_content = b"fake mp3 content";
        std::fs::write(temp_file.path(), test_content).expect("Failed to write test content");
        
        // Manually mark job as completed with MP3 path
        {
            let mut jobs = get_job_store().write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = "completed".to_string();
                job.mp3_path = Some(temp_file.path().to_path_buf());
            }
        }
        
        let result = get_mp3_file(&job_id).await;
        assert!(result.is_ok());
        
        let content = result.expect("Expected file content");
        assert_eq!(content, test_content);
    }

    #[tokio::test]
    async fn test_job_error_handling() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        let job_id = start_conversion(url).await.expect("Expected job ID");
        
        let error_message = "Test error message";
        
        // Manually set job to error state
        {
            let mut jobs = get_job_store().write().await;
            if let Some(job) = jobs.get_mut(&job_id) {
                job.status = "error".to_string();
                job.error = Some(error_message.to_string());
            }
        }
        
        let status = get_job_status(&job_id).await.expect("Expected job status");
        assert_eq!(status.status, "error");
        assert_eq!(status.message, error_message);
    }

    #[tokio::test]
    async fn test_concurrent_job_creation() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        
        // Create multiple jobs concurrently
        let tasks: Vec<_> = (0..5)
            .map(|_| {
                let url = url.clone();
                tokio::spawn(async move { start_conversion(url).await })
            })
            .collect();
        
        let results = futures::future::join_all(tasks).await;
        
        // All jobs should be created successfully
        let mut job_ids = Vec::new();
        for result in results {
            let job_id = result.expect("Task should complete").expect("Job should be created");
            job_ids.push(job_id);
        }
        
        // All job IDs should be unique
        job_ids.sort();
        job_ids.dedup();
        assert_eq!(job_ids.len(), 5, "All job IDs should be unique");
        
        // All jobs should be retrievable
        for job_id in job_ids {
            let status = get_job_status(&job_id).await.expect("Should get status");
            assert_eq!(status.status, "processing");
        }
    }

    #[tokio::test]
    async fn test_start_conversion_with_timeout() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string();
        
        // Test that start_conversion completes within reasonable time
        let result = timeout(Duration::from_secs(5), start_conversion(url)).await;
        assert!(result.is_ok(), "start_conversion should complete within timeout");
        
        let job_id = result.expect("Timeout").expect("Job creation");
        assert!(!job_id.is_empty());
    }
}