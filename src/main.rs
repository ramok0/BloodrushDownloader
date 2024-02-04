use std::{env, path::PathBuf, str::FromStr, time::Duration};

use registery::Registery;
use windows::core::s;

pub mod registery;
pub mod privileges;
pub mod downloader;
pub mod manifest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    if privileges::is_privileged().unwrap_or(false) == false {
        println!("You need to run this program as an administrator");
        //TODO: Demander les privilèges

        return Ok(());
    }

    // Test if bloodrush is downloaded

    let program_files = env::var("PROGRAMFILES")?;

    let bloodrush_root_path = PathBuf::from_str(&program_files)?.join("Bloodrush");

    if bloodrush_root_path.exists() && bloodrush_root_path.join("Hellscape").join("Binaries").join("Win64").join("Hellscape-Win64-Shipping.exe").exists() {
        println!("Bloodrush is already installed, do you want to reinstall it? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim() != "y" {
            return Ok(());
        }

        std::fs::remove_dir_all(bloodrush_root_path)?;

        println!("Deleted old installation successfully");
    }

    // Get manifest

    let manifest = manifest::Manifest::from_url(manifest::MANIFEST_URL).await?;

    // Ask for which mirror to use

    println!("Choose a mirror to download BloodRush from:");

    for (i, mirror) in manifest.mirrors.iter().enumerate() {
        if let Some(note) = mirror.note.as_ref() {
            println!("{}: {} ({})", i, mirror.name, note);
        } else {
            println!("{}: {}", i, mirror.name);
        }
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let mirror_index = input.trim().parse::<usize>()?;
    let mirror = manifest.mirrors.get(mirror_index).ok_or("Invalid mirror index")?;

    println!("Downloading BloodRush from {}", mirror.url);

    // Get download path

    let download_path = env::temp_dir().join("bloodrush.zip");
    if download_path.exists() {
        std::fs::remove_file(&download_path)?;
    }
    
    // Download bloodrush

    let downloader = downloader::Downloader::new();

    //send the download request to the downloader 
    downloader.download(&mirror.url, &download_path);
    
    // Wait for the download to finish

    loop {
        {
        let state = downloader.state.read().unwrap();
        if let Some(state) = &*state {
            match state.state {
                downloader::DownloadStatus::Downloading => {
                    println!("Downloading... {}%, {} bytes per second", (state.downloaded as f64 / state.total as f64) * 100.0, state.speed);
                },
                downloader::DownloadStatus::Error => {
                    println!("An error occured while downloading Bloodrush");
                    return Err("Failed to download BloodRush.".into());
                },
                downloader::DownloadStatus::Finished => {
                    println!("Download finished");
                    break;
                }
            }
        }
    }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }


    // Extract bloodrush

    // Install Bloodrush

    //Create a shortcut

    // Check integrity

    Ok(())
}