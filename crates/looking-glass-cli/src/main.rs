//! looking-glass-cli - CLI binary for Looking Glass launcher (lg-launch)

mod launcher;
mod profiles;

use clap::{Parser, ArgAction};
use launcher::GameLauncher;
use looking_glass_config::{config_dir, ConfigManager};
use profiles::{LauncherConfig, ProfileManager, ModProfile, Profile};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "lg-launch",
    version,
    about = "Looking Glass Game Launcher with mod support",
    long_about = "Launch the Looking Glass game with mod profiles and optional ReShade injection.\n\n\
    Mod profiles:\n  vanilla       - Launch vanilla (no mods)\n  cheats        - Launch with cheats mod\n  button-glyphs - Launch with button glyphs mod\n  <custom>      - Launch with custom profile"
)]
struct Cli {
    /// Mod profile to use (vanilla, cheats, button-glyphs, or custom profile name)
    #[arg(short, long, value_name = "PROFILE", default_value = "default")]
    profile: String,

    /// Enable ReShade injection
    #[arg(long)]
    reshade: bool,

    /// Path to ReShade DLL (overrides config)
    #[arg(long, value_name = "PATH")]
    reshade_path: Option<PathBuf>,

    /// Launch vanilla (no mods)
    #[arg(long)]
    no_mods: bool,

    /// List available mod profiles
    #[arg(long, action = ArgAction::SetTrue)]
    list_profiles: bool,

    /// Install a mod from archive file
    #[arg(long, value_name = "PATH")]
    install_mod: Option<PathBuf>,

    /// Game ID for mod operations (default: Alice)
    #[arg(long, value_name = "GAME_ID", default_value = "Alice")]
    game_id: String,

    /// Additional arguments to pass to the game
    #[arg(last = true, value_name = "ARGS")]
    game_args: Vec<String>,

    /// Environment variables to set (KEY=VAL format, can be used multiple times)
    #[arg(long, value_name = "KEY=VAL")]
    env: Vec<String>,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    /// Config file path
    #[arg(short, long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("looking_glass={},lg_launch={}", log_level, log_level))
        .init();

    // Load config
    let config_dir = cli.config.unwrap_or_else(config_dir);
    let launcher_config = LauncherConfig::load(&config_dir)?;

    // Create launcher
    let launcher = GameLauncher::new(launcher_config.clone());

    // Handle --list-profiles
    if cli.list_profiles {
        list_profiles(&launcher);
        return Ok(());
    }

    // Handle --install-mod
    if let Some(archive_path) = cli.install_mod {
        install_mod(&launcher, &archive_path, &cli.game_id)?;
        return Ok(());
    }

    // Parse extra environment variables
    let mut extra_env = HashMap::new();
    for env_pair in cli.env {
        if let Some((key, value)) = env_pair.split_once('=') {
            extra_env.insert(key.to_string(), value.to_string());
        } else {
            eprintln!("Warning: Invalid environment variable format: {} (expected KEY=VAL)", env_pair);
        }
    }

    // Determine profile
    let profile_name = if cli.no_mods {
        "vanilla"
    } else {
        &cli.profile
    };

    // Build launch options
    let options = launcher.build_options(
        profile_name,
        cli.game_args,
        extra_env,
        cli.no_mods,
        cli.reshade,
        cli.reshade_path,
    )?;

    // Launch the game
    launcher.launch(options)?;

    Ok(())
}

fn list_profiles(launcher: &GameLauncher) {
    println!("Available mod profiles:");
    println!();
    for profile in launcher.list_profiles() {
        let builtin_marker = if profile.name == "vanilla" || profile.name == "cheats" || profile.name == "button-glyphs" {
            " (builtin)"
        } else {
            ""
        };
        println!("  {}{}", profile.name, builtin_marker);
        if let Some(desc) = &profile.description {
            println!("    {}", desc);
        }
        if !profile.mod_ids.is_empty() {
            println!("    Mods: {}", profile.mod_ids.join(", "));
        } else {
            println!("    Mods: (none)");
        }
        println!();
    }
}

fn install_mod(launcher: &GameLauncher, archive_path: &PathBuf, game_id: &str) -> anyhow::Result<()> {
    if !archive_path.exists() {
        return Err(anyhow::anyhow!("Archive not found: {}", archive_path.display()));
    }

    println!("Installing mod from: {}", archive_path.display());
    let installed_mod = launcher.install_mod(archive_path, game_id)?;
    println!("Successfully installed: {} v{}", installed_mod.manifest.id, installed_mod.manifest.version);
    println!("Mod path: {}", installed_mod.path.display());

    Ok(())
}