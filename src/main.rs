use std::{collections::hash_map::DefaultHasher, env, hash::{Hash, Hasher}, io::Cursor, path::PathBuf, str::FromStr, time::Duration};
use mslnk::ShellLink;


pub mod registery;
pub mod privileges;
pub mod downloader;
pub mod manifest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    if privileges::is_privileged().unwrap_or(false) == false {
        println!("You need to run this program as an administrator");
        //TODO: Demander les privilèges
        tokio::time::sleep(Duration::from_secs(5)).await;
        return Ok(());
    }

    // Test if bloodrush is downloaded

    let program_files = env::var("PROGRAMFILES")?;

    let bloodrush_root_path = PathBuf::from_str(&program_files)?.join("Bloodrush");
    let bloodrush_game_path = bloodrush_root_path.join("Hellscape");
    let bloodrush_exe_path = bloodrush_game_path.join("Binaries").join("Win64").join("Hellscape-Win64-Shipping.exe");

    if bloodrush_root_path.exists() && bloodrush_exe_path.exists() {
        println!("Bloodrush is already installed, do you want to reinstall it? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim() != "y" {
            return Ok(());
        }

        std::fs::remove_dir_all(&bloodrush_root_path)?;

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
                    println!("Downloading... {}%, {} Mbps", ((state.downloaded as f64 / state.total as f64) * 100.0).round(), (state.speed * 1e-6).round());
                },
                downloader::DownloadStatus::Error => {
                    println!("An error occured while downloading Bloodrush");
                    tokio::time::sleep(Duration::from_secs(5)).await;
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

    println!("Extracting Bloodrush... Please wait !");

    let archive_content = std::fs::read(&download_path)?;
    zip_extract::extract(Cursor::new(archive_content), &bloodrush_root_path, true)?;

    println!("Archive extracted successfully");
    std::fs::remove_file(&download_path)?;

    // Check integrity

    let exe_content = std::fs::read(&bloodrush_exe_path)?;
    let mut hasher = DefaultHasher::new();

    exe_content.hash(&mut hasher);

    if hasher.finish() != manifest.current_exe_hash {
        println!("The exe Bloodrush is corrupted, please try again, current hash : {}, expected hash : {}", hasher.finish(), manifest.current_exe_hash);
        tokio::time::sleep(Duration::from_secs(5)).await;
        return Ok(());
    }

    let pak_content = std::fs::read(bloodrush_game_path.join("Content").join("Paks").join("Hellscape-WindowsNoEditor.pak"))?;

    let mut hasher = DefaultHasher::new();
    pak_content.hash(&mut hasher);

    if hasher.finish() != manifest.current_pak_hash {
        println!("The game pak is corrupted, please try again, current hash: {}, expected hash : {}", hasher.finish(), manifest.current_pak_hash);
        tokio::time::sleep(Duration::from_secs(5)).await;
        return Ok(())
    }

    println!("Game integrity OK");

    //Create a shortcut

    let sl = ShellLink::new(bloodrush_exe_path)?;
    
    let user_profile = env::var("USERPROFILE")?;
    let shortcut_path = PathBuf::from_str(&user_profile)?.join("Desktop").join("Blood Rush.lnk");
    sl.create_lnk(&shortcut_path)?;

    println!("BloodRush has been successfully installed !");

    Ok(())
}