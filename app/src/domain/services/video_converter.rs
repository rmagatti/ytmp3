use leptos::prelude::*;
use serde::{Deserialize, Serialize};

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
        use crate::domain::services::video_converter::server::{
            is_valid_youtube_url, start_conversion,
        };

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

#[cfg(feature = "ssr")]
pub mod server {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::process::Stdio;
    use leptos::logging::log;
    use tempfile::TempDir;
    use tokio::process::Command;
    use uuid::Uuid;

    use crate::domain::services::video_converter::ConvertResponse;

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
        let temp_dir = tempfile::Builder::new()
            .prefix("ytmp3_")
            .tempdir_in("/home/app")?;

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
          log!("Starting conversion for job {}: {}", job_id, url);
          
          let temp_dir_path = {
              let jobs = JOB_STORE.read().await;
              if let Some(job) = jobs.get(&job_id) {
                  job.temp_dir.path().to_path_buf()
              } else {
                  log!("Job {} not found in store", job_id);
                  return;
              }
          };

          // Multiple retry strategies
          let strategies = vec![
              // Strategy 1: Android client (Docker-optimized)
              vec![
                  "--extractor-args", "youtube:player_client=android",
                  "--user-agent", "com.google.android.youtube/17.31.35 (Linux; U; Android 11) gzip",
                  "--no-check-certificates",
              ],
              // Strategy 2: Android TV client (often works well in containers)
              vec![
                  "--extractor-args", "youtube:player_client=android_embedded",
                  "--user-agent", "com.google.android.youtube/17.31.35 (Linux; U; Android 11) gzip",
              ],
              // Strategy 3: iOS client
              vec![
                  "--extractor-args", "youtube:player_client=ios",
                  "--user-agent", "com.google.ios.youtube/17.31.4 (iPhone14,3; U; CPU iOS 15_6 like Mac OS X)",
              ],
              // Strategy 4: Web client without cookies (Docker-safe)
              vec![
                  "--extractor-args", "youtube:player_client=web",
                  "--user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                  "--add-header", "Accept-Language:en-US,en;q=0.9",
              ],
              // Strategy 5: Legacy method with Docker optimizations
              vec![
                  "--extractor-args", "youtube:player_client=web",
                  "--compat-options", "prefer-legacy-http-handler",
                  "--no-check-certificates",
                  "--prefer-insecure",
              ],
              // Strategy 6: Minimal approach for containers
              vec![
                  "--extractor-args", "youtube:player_client=mediaconnect",
                  "--socket-timeout", "30",
              ],
          ];

          let mut final_mp3_path = None;
          let mut last_error = String::new();

          for (attempt, strategy) in strategies.iter().enumerate() {
              log!("Job {} attempt {} with strategy: {:?}", job_id, attempt + 1, strategy);

              let mut cmd = Command::new("yt-dlp");
              cmd.arg(&url)
                  .arg("-x")
                  .arg("--audio-format")
                  .arg("mp3")
                  .arg("--audio-quality")
                  .arg("192K")
                  .arg("-o")
                  .arg("%(title)s.%(ext)s")
                  .arg("--restrict-filenames")
                  .current_dir(&temp_dir_path)
                  .arg("--retries")
                  .arg("2")
                  .arg("--retry-sleep")
                  .arg("3")
                  .stdout(Stdio::piped())
                  .stderr(Stdio::piped());

              // Add strategy-specific arguments
              for arg in strategy {
                  cmd.arg(arg);
              }

              // Add common anti-detection measures for non-android strategies
              if !strategy.contains(&"youtube:player_client=android") {
                  cmd.arg("--sleep-interval")
                      .arg("1")
                      .arg("--max-sleep-interval")
                      .arg("3");
              }

              let download_result = cmd.output().await;

              match download_result {
                  Ok(output) => {
                      log!("Job {} attempt {} completed with status: {}", job_id, attempt + 1, output.status);
                      log!("Job {} attempt {} stdout: {}", job_id, attempt + 1, String::from_utf8_lossy(&output.stdout));
                      
                      if output.status.success() {
                          log!("Job {} attempt {} succeeded", job_id, attempt + 1);
                          
                          // Look for MP3 file
                          if let Ok(mut entries) = tokio::fs::read_dir(&temp_dir_path).await {
                              while let Ok(Some(entry)) = entries.next_entry().await {
                                  let path = entry.path();
                                  if path.extension().and_then(|s| s.to_str()) == Some("mp3") {
                                      final_mp3_path = Some(path);
                                      break;
                                  }
                              }
                          }
                          
                          if final_mp3_path.is_some() {
                              break; // Success!
                          }
                      } else {
                          let error_msg = String::from_utf8_lossy(&output.stderr);
                          last_error = error_msg.to_string();
                          log!("Job {} attempt {} failed: {}", job_id, attempt + 1, error_msg);
                          
                          // If it's a rate limit or bot detection, wait before next attempt
                          if error_msg.contains("Sign in to confirm") || 
                             error_msg.contains("rate limit") ||
                             error_msg.contains("429") {
                              tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                          }
                      }
                  }
                  Err(e) => {
                      last_error = format!("Command execution failed: {}", e);
                      log!("Job {} attempt {} command failed: {}", job_id, attempt + 1, e);
                  }
              }

              // Small delay between attempts
              if attempt < strategies.len() - 1 {
                  tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
              }
          }

          // Update job status based on results
          let mut jobs = JOB_STORE.write().await;
          if let Some(job) = jobs.get_mut(&job_id) {
              if let Some(mp3_path) = final_mp3_path {
                  job.status = "completed".to_string();
                  job.mp3_path = Some(mp3_path);
                  log!("Job {} completed successfully", job_id);
              } else {
                  job.status = "error".to_string();
                  
                  // Provide user-friendly error message
                  let user_friendly_error = if last_error.contains("Sign in to confirm you're not a bot") {
                      "YouTube is currently blocking automated downloads. This is temporary - please try again in 10-15 minutes, or try a different video.".to_string()
                  } else if last_error.contains("Failed to extract any player response") {
                      "YouTube has updated their protection. Please try again in a few minutes, or contact support if the issue persists.".to_string()
                  } else if last_error.contains("Video unavailable") {
                      "This video is unavailable. It may be private, deleted, or region-restricted.".to_string()
                  } else if last_error.contains("age-restricted") || last_error.contains("age_restricted") {
                      "This video is age-restricted and cannot be downloaded without authentication.".to_string()
                  } else if last_error.contains("rate limit") || last_error.contains("too many requests") || last_error.contains("HTTP Error 429") {
                      "YouTube is rate limiting requests. Please wait a few minutes before trying again.".to_string()
                  } else if last_error.contains("premieres in") {
                      "This video is a premiere that hasn't started yet. Please wait until it's available.".to_string()
                  } else if last_error.contains("live stream") {
                      "Live streams cannot be downloaded. Please wait until the stream ends or try a regular video.".to_string()
                  } else {
                      format!("Download failed after multiple attempts. Last error: {}", 
                          last_error.lines().take(2).collect::<Vec<_>>().join(" "))
                  };
                  
                  job.error = Some(user_friendly_error);
                  log!("Job {} failed with error: {}", job_id, job.error.as_ref().unwrap());
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
