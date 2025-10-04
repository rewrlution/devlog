use color_eyre::Result;

#[derive(clap::Subcommand)]
pub enum ConfigSubcommand {
    /// Set base path for dev logs
    Path,
    /// Configure cloud sync provider
    Sync {
        /// Cloud sync provider (azure, aws, gcp, etc)
        provider: String,
    },
    /// Open config file in editor
    Edit,
    /// Show current configuration
    Show,
    /// Reset configuration to defaults
    Reset,
}

pub fn execute(subcmd: Option<ConfigSubcommand>) -> Result<()> {
    match subcmd {
        None => println!("Configure devlog interactively"),
        Some(ConfigSubcommand::Path) => println!("Configure path"),
        Some(ConfigSubcommand::Sync { provider }) => {
            println!("Configure sync provider: {}", provider)
        }
        Some(ConfigSubcommand::Edit) => println!("Configure edit"),
        Some(ConfigSubcommand::Show) => println!("Current configuration"),
        Some(ConfigSubcommand::Reset) => println!("Reset to default settings"),
    }

    Ok(())
}
