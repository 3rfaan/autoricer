use crate::{
    clone, error, info, prompt, success, tip,
    utils::{helper_functions::*, types::*},
    warning,
};
use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub fn installation_prompt() -> io::Result<Installation> {
    let mut input: String;

    tip!("This installer will copy the config files from this repo: https://github.com/3rfaan/arch-everforest\n\
          Make sure you've installed those programs to get the best experience.");
    warning!("==> This installer will modify directories inside your ~/.config directory");

    loop {
        prompt!("Do you want to proceed? [y/N]");

        input = read_input()?;

        match parse_input(&input) {
            UserInput::Yes => return Ok(Installation::Proceed),
            UserInput::No => return Ok(Installation::Exit),
            UserInput::Other => prompt!("==> Please enter [y]es or [n]o!"),
        }
    }
}

// Clones Github repo into ~/Downloads/arch-everforest
pub fn clone_repo(config_path: &Path, repo_path: &Path) -> io::Result<DownloadStatus> {
    const URL: &str = "https://github.com/3rfaan/dotfiles";

    info!("Cloning into https://github.com/3rfaan/dotfiles...");

    if !config_path.exists() {
        fs::create_dir_all(config_path)?;
    }

    if repo_path.exists() {
        return Ok(DownloadStatus::Existing);
    }

    clone!(URL, repo_path)?;

    Ok(DownloadStatus::Success)
}

// Delete directories and files which are not needed to moved to ~/.config directory
pub fn cleanup_repo(paths: &Paths) -> io::Result<()> {
    let entries_to_delete: &[&str] = &[
        "arch_wallpaper.jpg",
        "preview_1.png",
        "preview_2.png",
        "preview_3.png",
        "preview_4.png",
        ".git",
        "README.md",
        "zsh",
    ];

    info!("Removing some directories and files which are not needed to be moved to ~/.config...");

    cleanup(paths, entries_to_delete)?;

    Ok(())
}

// Creates backup of all files and directories inside ~/.config and puts it inside ~/Documents/config_backup
pub fn create_backup(config_path: &Path, documents_path: &Path) -> io::Result<BackupStatus> {
    let backup_path: PathBuf = documents_path.join("config_backup");

    info!("Creating backup of your current ~/.config directory...");

    if backup_path.exists() {
        return Ok(BackupStatus::Existing);
    } else {
        fs::create_dir(&backup_path)?;
    }

    if let Err(error) = copy_recursively(config_path, backup_path) {
        error!(
            "Could not create backup directory at ~/Documents/backup",
            error
        );

        loop {
            prompt!("The theme can still be installed. Do you want to continue? [y/N]");

            let input: String = read_input()?;

            match parse_input(&input) {
                UserInput::Yes => return Ok(BackupStatus::NoBackup),
                UserInput::No => return Err(error),
                UserInput::Other => prompt!("==> Please enter [y]es or [n]o!"),
            }
        }
    }

    Ok(BackupStatus::Created)
}

// Copy directories from ~/Downloads/arch-everforest to ~/.config recursively
pub fn copy_config_dirs_recursively(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;

    info!("Copying directories from ~/Downloads/arch-everforest to ~/.config...");

    copy_recursively(src, dest)?;

    Ok(())
}

// Prompt for changing settings inside ~/.config/hypr/hyprland.conf
pub fn change_settings(hypr_config: &Path) -> io::Result<HyprConfig> {
    let mut input: String;

    let mut change_kb_layout: bool;
    let mut layout_code: String = String::from("us");

    let change_nvidia_env_vars: bool;

    loop {
        prompt!("Keyboard layout is currently set to [us]. Would you like to change it? [y/N]");

        input = read_input()?;

        match parse_input(&input) {
            UserInput::Yes => {
                change_kb_layout = true;
                break;
            }
            UserInput::No => {
                change_kb_layout = false;
                break;
            }
            UserInput::Other => prompt!("==> Please enter [y]es or [n]o!"),
        }
    }

    if change_kb_layout {
        match get_kb_layout_code() {
            Ok(KBLayout::Change(code)) => layout_code = code,
            Ok(KBLayout::Default) => change_kb_layout = false,
            Err(error) => return Err(error),
        }
    }

    loop {
        prompt!("Are you using a NVIDIA graphics card? [y/N]");

        input = read_input()?;

        match parse_input(&input) {
            UserInput::Yes => {
                change_nvidia_env_vars = true;
                break;
            }
            UserInput::No => {
                change_nvidia_env_vars = false;
                break;
            }
            UserInput::Other => prompt!("==> Please enter [y]es or [n]o!"),
        }
    }

    if !change_kb_layout && !change_nvidia_env_vars {
        return Ok(HyprConfig::Default);
    }

    update_hypr_config(
        hypr_config,
        change_kb_layout,
        change_nvidia_env_vars,
        &layout_code,
    )?;

    Ok(HyprConfig::Modified)
}

// Helper function for `change_settings()` to modify Hyprland config file
fn update_hypr_config(
    hypr_config: &Path,
    change_kb_layout: bool,
    change_nvidia_env_vars: bool,
    layout_code: &str,
) -> io::Result<HyprConfig> {
    if layout_code == "us" && !change_nvidia_env_vars {
        return Ok(HyprConfig::Default);
    }

    // Path to Hyprland config file
    let hypr_config_file: File = File::open(hypr_config)?;
    let hypr_config_reader: BufReader<File> = BufReader::new(hypr_config_file);

    let temp_file_path: &Path = Path::new("./hyprland.conf");
    let temp_file: File = File::create(temp_file_path)?;
    let mut temp_file_stream: BufWriter<File> = BufWriter::new(temp_file);

    let old_layout: &str = "kb_layout = us";
    let new_layout: String = format!("kb_layout = {}", layout_code);

    static NVIDIA_ENV_VARS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#(env = .+)$").unwrap());

    info!("Modifying Hypr config with your settings...");

    for line in hypr_config_reader.lines() {
        let mut line: String = line?;

        if change_kb_layout && line.contains(old_layout) {
            line = line.replace(old_layout, &new_layout);

            println!(
                "{} {}",
                "==> Changed the following line in Hypr config file:".green(),
                line.trim().green().bold()
            );
        }

        if change_nvidia_env_vars && NVIDIA_ENV_VARS_RE.is_match(&line) {
            line = NVIDIA_ENV_VARS_RE.replace(&line, "$1").to_string();
        }

        temp_file_stream.write_all(line.as_bytes())?;
        temp_file_stream.write_all(b"\n")?;
    }

    temp_file_stream.flush()?;

    if temp_file_path.exists() {
        fs::copy(temp_file_path, hypr_config)?;
        fs::remove_file(temp_file_path)?;

        success!("==> Copied new Hypr config file to ~/.config/Hypr/hyprland.conf");
        success!("==> Removed temporary file");
    }

    Ok(HyprConfig::Modified)
}

pub fn install_cli_utilities(home_path: &Path, config_path: &Path) -> io::Result<DownloadStatus> {
    let zsh_path: PathBuf = home_path.join(".zsh");
    let yazi_path: PathBuf = config_path.join("yazi/plugins");

    info!("Installing CLI utilies");

    if !zsh_path.exists() {
        fs::create_dir_all(&zsh_path)?;
    }

    if !zsh_path.join("zsh-autosuggestions").exists() {
        clone!(
            "https://github.com/zsh-users/zsh-autosuggestions",
            &zsh_path.join("zsh-autosuggestions")
        )?;

        success!("==> Successfully cloned zsh-autosuggestions");
    }

    if !zsh_path.join("zsh-syntax-highlighting").exists() {
        clone!(
            "https://github.com/zsh-users/zsh-syntax-highlighting.git",
            &zsh_path.join("zsh-syntax-highlighting")
        )?;

        success!("==> Successfully cloned zsh-syntax-highlighting");
    }

    if !yazi_path.exists() {
        fs::create_dir_all(&yazi_path)?;
    }

    let yazi_packages: &[&str] = &[
        "dedukun/bookmarks",
        "dedukun/relative-motions",
        "yazi-rs/flavors:catppuccin-macchiato",
        "yazi-rs/plugins:full-border",
    ];

    if command_exists("ya") {
        for &package in yazi_packages {
            Command::new("ya")
                .arg("pack")
                .arg("-a")
                .arg(package)
                .output()?;
        }
    } else {
        warning!("Could not install ya packages for yazi. Make sure ya is installed and try to install them manually");
    }

    Ok(DownloadStatus::Success)
}

pub fn after_install(repo_path: &Path) -> io::Result<()> {
    info!("Removing ~/Downloads/arch-everforest repo");

    if repo_path.exists() {
        fs::remove_dir_all(repo_path)?;
    }

    success!("==> Removed repo successfully".green());

    info!("Installation succeeded! 🎉");

    tip!("Tip: After this installation you have to restart Hyprland by pressing <SUPER> + <SHIFT> + E");
    tip!("Tip: To change your wallpaper change the path to another picture inside ~/.config/hypr/hyprpaper.conf");

    Ok(())
}
