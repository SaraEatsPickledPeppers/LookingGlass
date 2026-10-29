use clap::{Parser, Subcommand, Args};
use anyhow::Result;
use tracing::{info, debug, Level};
use tracing_subscriber::{fmt, EnvFilter};

mod config;
mod commands;
mod output;

use config::{Config, ConfigManager};
use output::{Output, OutputFormat};

#[derive(Parser, Debug)]
#[command(
    name = "lg",
    version,
    about = "Looking Glass - Mod manager and launcher for Alice games",
    long_about = "Looking Glass is a mod manager and launcher for Alice and Alice: Madness Returns.\nIt handles mod installation, profiles, and game launching with mod support."
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true, value_name = "PATH")]
    config: Option<String>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Output as JSON
    #[arg(long, global = true)]
    json_output: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch a game with mods
    Launch(LaunchArgs),

    /// List games, mods, or profiles
    List(ListArgs),

    /// Mod management commands
    #[command(subcommand)]
    Mod(ModCommands),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Game registry management
    #[command(subcommand)]
    Registry(RegistryCommands),
}

#[derive(Args, Debug)]
struct LaunchArgs {
    /// Game ID to launch
    game_id: String,

    /// Mod profile to use
    #[arg(long, value_name = "NAME")]
    mod_profile: Option<String>,

    /// Additional mods to enable (can be used multiple times)
    #[arg(long, value_name = "MOD_ID")]
    mod: Vec<String>,

    /// Additional arguments to pass to the game
    #[arg(long, value_name = "ARGS")]
    args: Vec<String>,

    /// Environment variables to set (KEY=VAL format, can be used multiple times)
    #[arg(long, value_name = "KEY=VAL")]
    env: Vec<String>,
}

#[derive(Args, Debug)]
struct ListArgs {
    /// Show only installed games
    #[arg(long)]
    installed: bool,

    /// Filter by platform
    #[arg(long, value_name = "PLATFORM")]
    platform: Option<String>,

    /// Search query
    #[arg(long, value_name = "QUERY")]
    search: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ModCommands {
    /// Install a mod from file or URL
    Install(ModInstallArgs),

    /// Uninstall a mod
    Uninstall(ModUninstallArgs),

    /// Enable a mod
    Enable(ModEnableArgs),

    /// Disable a mod
    Disable(ModDisableArgs),

    /// List installed mods
    List(ModListArgs),

    /// Profile management
    #[command(subcommand)]
    Profile(ModProfileCommands),
}

#[derive(Args, Debug)]
struct ModInstallArgs {
    /// Path or URL to mod archive
    source: String,

    /// Game ID to install for
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,

    /// Enable mod after installation
    #[arg(long)]
    enable: bool,
}

#[derive(Args, Debug)]
struct ModUninstallArgs {
    /// Mod ID to uninstall
    mod_id: String,

    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,
}

#[derive(Args, Debug)]
struct ModEnableArgs {
    /// Mod ID to enable
    mod_id: String,

    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,

    /// Profile name
    #[arg(long, value_name = "NAME")]
    profile: Option<String>,
}

#[derive(Args, Debug)]
struct ModDisableArgs {
    /// Mod ID to disable
    mod_id: String,

    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,

    /// Profile name
    #[arg(long, value_name = "NAME")]
    profile: Option<String>,
}

#[derive(Args, Debug)]
struct ModListArgs {
    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,

    /// Profile name
    #[arg(long, value_name = "NAME")]
    profile: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ModProfileCommands {
    /// Create a new mod profile
    Create(ModProfileCreateArgs),

    /// Delete a mod profile
    Delete(ModProfileDeleteArgs),

    /// Switch active mod profile
    Switch(ModProfileSwitchArgs),

    /// List mod profiles
    List(ModProfileListArgs),
}

#[derive(Args, Debug)]
struct ModProfileCreateArgs {
    /// Profile name
    name: String,

    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,

    /// Clone from existing profile
    #[arg(long, value_name = "PROFILE")]
    clone: Option<String>,
}

#[derive(Args, Debug)]
struct ModProfileDeleteArgs {
    /// Profile name
    name: String,

    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,
}

#[derive(Args, Debug)]
struct ModProfileSwitchArgs {
    /// Profile name
    name: String,

    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,
}

#[derive(Args, Debug)]
struct ModProfileListArgs {
    /// Game ID
    #[arg(long, value_name = "GAME_ID")]
    game: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Show configuration
    Show(ConfigShowArgs),

    /// Set a configuration value
    Set(ConfigSetArgs),

    /// Get a configuration value
    Get(ConfigGetArgs),

    /// Reset configuration
    Reset(ConfigResetArgs),
}

#[derive(Args, Debug)]
struct ConfigShowArgs {
    /// Show specific key
    #[arg(long, value_name = "KEY")]
    key: Option<String>,
}

#[derive(Args, Debug)]
struct ConfigSetArgs {
    /// Configuration key
    key: String,

    /// Configuration value
    value: String,
}

#[derive(Args, Debug)]
struct ConfigGetArgs {
    /// Configuration key
    key: String,
}

#[derive(Args, Debug)]
struct ConfigResetArgs {
    /// Reset specific key (otherwise reset all)
    #[arg(long, value_name = "KEY")]
    key: Option<String>,
}

#[derive(Subcommand, Debug)]
enum RegistryCommands {
    /// Scan for installed games
    Scan(RegistryScanArgs),

    /// Add a game to registry
    Add(RegistryAddArgs),

    /// Remove a game from registry
    Remove(RegistryRemoveArgs),
}

#[derive(Args, Debug)]
struct RegistryScanArgs {
    /// Force rescan
    #[arg(long)]
    force: bool,
}

#[derive(Args, Debug)]
struct RegistryAddArgs {
    /// Game ID
    game_id: String,

    /// Game name
    #[arg(long, value_name = "NAME")]
    name: String,

    /// Executable path
    #[arg(long, value_name = "PATH")]
    exe: String,

    /// Platform
    #[arg(long, value_name = "PLATFORM")]
    platform: Option<String>,

    /// Default launch arguments
    #[arg(long, value_name = "ARGS")]
    args: Vec<String>,

    /// Default environment variables
    #[arg(long, value_name = "KEY=VAL")]
    env: Vec<String>,
}

#[derive(Args, Debug)]
struct RegistryRemoveArgs {
    /// Game ID to remove
    game_id: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else if cli.quiet {
        Level::ERROR
    } else {
        Level::INFO
    };

    let filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    debug!("CLI parsed: {:?}", cli);

    // Determine output format
    let output_format = if cli.json_output {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    };

    let output = Output::new(output_format);

    // Load configuration
    let mut config_manager = ConfigManager::new(cli.config.as_deref())?;
    let config = config_manager.load()?;

    // Dispatch command
    let result = match cli.command {
        Commands::Launch(args) => commands::launch::execute(args, &config, &output),
        Commands::List(args) => commands::list::execute(args, &config, &output),
        Commands::Mod(cmd) => match cmd {
            ModCommands::Install(args) => commands::mod_install::execute(args, &config, &output),
            ModCommands::Uninstall(args) => commands::mod_uninstall::execute(args, &config, &output),
            ModCommands::Enable(args) => commands::mod_enable::execute(args, &config, &output),
            ModCommands::Disable(args) => commands::mod_disable::execute(args, &config, &output),
            ModCommands::List(args) => commands::mod_list::execute(args, &config, &output),
            ModCommands::Profile(cmd) => match cmd {
                ModProfileCommands::Create(args) => commands::mod_profile_create::execute(args, &config, &output),
                ModProfileCommands::Delete(args) => commands::mod_profile_delete::execute(args, &config, &output),
                ModProfileCommands::Switch(args) => commands::mod_profile_switch::execute(args, &config, &output),
                ModProfileCommands::List(args) => commands::mod_profile_list::execute(args, &config, &output),
            },
        },
        Commands::Config(cmd) => match cmd {
            ConfigCommands::Show(args) => commands::config_show::execute(args, &config, &output),
            ConfigCommands::Set(args) => commands::config_set::execute(args, &mut config_manager, &output),
            ConfigCommands::Get(args) => commands::config_get::execute(args, &config, &output),
            ConfigCommands::Reset(args) => commands::config_reset::execute(args, &mut config_manager, &output),
        },
        Commands::Registry(cmd) => match cmd {
            RegistryCommands::Scan(args) => commands::registry_scan::execute(args, &config, &output),
            RegistryCommands::Add(args) => commands::registry_add::execute(args, &mut config_manager, &output),
            RegistryCommands::Remove(args) => commands::registry_remove::execute(args, &mut config_manager, &output),
        },
    };

    if let Err(e) = result {
        output.error(&e.to_string());
        std::process::exit(1);
    }

    Ok(())
}