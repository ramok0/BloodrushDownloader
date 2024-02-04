use futures_util::StreamExt;
use std::{
    io::Write,
    sync::{Arc, RwLock},
    time::Instant,
};

pub struct Downloader {
    client: reqwest::Client,
    pub state: Arc<RwLock<Option<DownloadState>>>,
}

pub struct DownloadState {
    pub total: u64,
    pub downloaded: u64,
    pub started_at: Instant,
    last_speed_check: Instant,
    last_speed_check_downloaded: u64,
    pub speed: f64, // bytes per second on the last 5s, 0 if not enough data
    pub state: DownloadStatus,
}

#[derive(Debug, PartialEq)]
pub enum DownloadStatus {
    Downloading,
    Error,
    Finished,
}

impl Downloader {
    pub fn new() -> Self {
        Downloader {
            client: reqwest::Client::new(),
            state: Arc::new(RwLock::new(None)),
        }
    }

    pub fn download(&self, url: &str, path: &std::path::Path) {
        let state = self.state.clone();
        let client = self.client.clone();
        let url = url.to_string();
        let path = path.to_path_buf();
        tokio::task::spawn(async move {
            let response = client.get(url).send().await;

            if response.is_err() {
                dbg!(response);
                let mut state = state.write().unwrap();
                *state = Some(DownloadState {
                    total: 0,
                    downloaded: 0,
                    started_at: Instant::now(),
                    last_speed_check: Instant::now(),
                    last_speed_check_downloaded: 0,
                    speed: 0.0,
                    state: DownloadStatus::Error,
                });

                return;
            }

            let response = response.unwrap();

            dbg!(response.status());

            

            let file = std::fs::File::create(path);

            if file.is_err() {
                dbg!(file);

                let mut state = state.write().unwrap();
                *state = Some(DownloadState {
                    total: 0,
                    downloaded: 0,
                    started_at: Instant::now(),
                    last_speed_check: Instant::now(),
                    last_speed_check_downloaded: 0,
                    speed: 0.0,
                    state: DownloadStatus::Error,
                });

                return;
            }

            let mut file = file.unwrap();
            let content_length = response.content_length().unwrap_or(0);

            dbg!(content_length);

            let mut stream = response.bytes_stream();

            println!("Got stream");

            let started_at = Instant::now();

            {
                let mut state = state.write().unwrap();
                *state = Some(DownloadState {
                    total: content_length,
                    downloaded: 0,
                    started_at,
                    last_speed_check: Instant::now(),
                    last_speed_check_downloaded: 0,
                    speed: 0.0,
                    state: DownloadStatus::Downloading,
                });
            }
            
            while let Some(item) = stream.next().await {
                if let Ok(chunk) = item {
             //       dbg!("got chunk");
                                   //mettre a jour "downloaded" et "speed" dans le state
                {
                    let mut state = state.write().unwrap();
                    let state = state.as_mut().unwrap();

                    state.downloaded += chunk.len() as u64;
                    state.last_speed_check_downloaded += chunk.len() as u64;
                    
                    state.speed = state.last_speed_check_downloaded as f64 / state.last_speed_check.elapsed().as_secs_f64();

                    if state.last_speed_check.elapsed().as_secs() >= 5 {
                        state.last_speed_check = Instant::now();
                        state.last_speed_check_downloaded = 0;
                    }
                }

                let file_write_result = file.write_all(&chunk);

                if file_write_result.is_err() {
                    dbg!(file_write_result);
                    let mut state = state.write().unwrap();
                    *state = Some(DownloadState {
                        total: content_length,
                        downloaded: 0,
                        started_at,
                        last_speed_check: Instant::now(),
                        last_speed_check_downloaded: 0,
                        speed: 0.0,
                        state: DownloadStatus::Error,
                    });
                    break;
                }
                }

 
            }
            {
                let mut state = state.write().unwrap();
                let state = state.as_mut().unwrap();
                if state.state == DownloadStatus::Downloading {
                    state.state = DownloadStatus::Finished;
                }
            }
        });
    }
}
