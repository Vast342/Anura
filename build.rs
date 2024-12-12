use std::{env, fs, io};
use std::path::Path;
use reqwest::blocking::get;
use std::process::Command;

fn main() {
    let nets_txt_path = "nets.txt";
    
    let net_name = match fs::read_to_string(nets_txt_path) {
        Ok(content) => content.trim().to_string(),
        Err(_) => panic!("Could not read the nets.txt file."),
    };

    let url = format!(
        "https://github.com/Vast342/anura-nets/releases/download/{}/{}.nets",
        net_name, net_name
    );

    let netfile_path = format!("src/nets/{}.nets", net_name);
    let netfile_path = Path::new(&netfile_path);

    if !netfile_path.exists() {
        println!("Downloading {} from GitHub...", net_name);
        download_file(&url, &netfile_path).expect("Failed to download the net file");
    } else {
        println!("Net file {} found locally.", net_name);
    }
    println!("cargo:rerun-if-changed={}", nets_txt_path);
    println!("cargo:rerun-if-changed={}", netfile_path.display());

    println!("cargo:netfile_path={}", netfile_path.display());
}

fn download_file(url: &str, dest: &Path) -> io::Result<()> {
    let response = get(url).expect("Failed to send GET request");

    if !response.status().is_success() {
        panic!("Failed to download file from {}: {}", url, response.status());
    }

    let mut file = fs::File::create(dest)?;
    let content = response.bytes().expect("Failed to read response body");
    io::copy(&mut content.as_ref(), &mut file)?;

    println!("Downloaded {} to {:?}", url, dest);
    Ok(())
}
