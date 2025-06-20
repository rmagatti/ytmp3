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
                        job.error = Some(format!("yt-dlp failed: {}", error_msg));
                    }
                }
            }
            Err(e) => {
                let mut jobs = JOB_STORE.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.status = "error".to_string();
                    job.error = Some(format!("Failed to execute yt-dlp: {}", e));
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
