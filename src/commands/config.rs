use crate::config::interactive;
use color_eyre::Result;

#[derive(clap::Subcommand)]
pub enum ConfigSubcommand {
    /// Set base path for dev logs
    Path,
    /// Configure storage provider
    Storage {
        /// Storage provider (local, azure)
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
        None => interactive::run_interactive_config(),
        Some(ConfigSubcommand::Path) => interactive::configure_path(),
        Some(ConfigSubcommand::Storage { provider }) => {
            interactive::configure_storage_provider(&provider)
        }
        Some(ConfigSubcommand::Edit) => interactive::edit_config(),
        Some(ConfigSubcommand::Show) => interactive::show_config(),
        Some(ConfigSubcommand::Reset) => interactive::reset_config(),
    }
}