use crate::config::interactive;
use color_eyre::Result;

#[derive(clap::Subcommand)]
pub enum ConfigSubcommand {
    /// Set base path for dev logs
    Path,
    /// Configure cloud sync provider
    Sync {
        /// Cloud sync provider (azure, aws, gcp)
        provider: Option<String>,
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
        None => interactive::run_interactive_config(),
        Some(ConfigSubcommand::Path) => interactive::configure_path(),
        Some(ConfigSubcommand::Sync { provider }) => {
            interactive::configure_sync_provider(provider.as_deref())
        }
        Some(ConfigSubcommand::Edit) => interactive::edit_config(),
        Some(ConfigSubcommand::Show) => interactive::show_config(),
        Some(ConfigSubcommand::Reset) => interactive::reset_config(),
    }
}