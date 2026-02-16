use clap::{Args, Parser, Subcommand};

/// Builder helper for calnoto
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Args, Debug, Clone)]
pub struct CmdBuildArgs {
    /// Build in debug mode
    #[clap(long)]
    pub debug: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
    /// Build the project
    Build(CmdBuildArgs),

    /// Build android APK
    BuildAndroid(CmdBuildArgs),

    /// Build for windows on linux
    #[cfg_attr(target_os = "windows", clap(skip))]
    BuildWindows(CmdBuildArgs),

    /// Run tests
    Test,

    /// Clean build artifacts
    Clean,
}
