//! `reth db settings` command for managing storage settings

use clap::{ArgAction, Parser, Subcommand};
use reth_db::open_db_read_only;
use reth_db_api::{database::Database, tables::Metadata, transaction::DbTx};
use reth_db_common::DbTool;
use reth_node_core::args::DatabaseArgs;
use reth_provider::{
    providers::ProviderNodeTypes, DBProvider, DatabaseProviderFactory, MetadataProvider,
    MetadataWriter, StorageSettings,
};
use reth_storage_api::metadata::keys::STORAGE_SETTINGS;
use std::path::Path;

use crate::common::AccessRights;

/// `reth db settings` subcommand
#[derive(Debug, Parser)]
pub struct Command {
    #[command(subcommand)]
    command: Subcommands,
}

impl Command {
    /// Returns database access rights required for the command.
    pub fn access_rights(&self) -> AccessRights {
        match self.command {
            Subcommands::Get => AccessRights::RO,
            Subcommands::Set(_) => AccessRights::RW,
        }
    }

    /// Returns `true` if this command only reads metadata from MDBX (no static files).
    pub const fn is_get(&self) -> bool {
        matches!(self.command, Subcommands::Get)
    }
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum Subcommands {
    /// Get current storage settings from database
    Get,
    /// Set storage settings in database
    #[clap(subcommand)]
    Set(SetCommand),
}

/// Set storage settings
#[derive(Debug, Clone, Copy, Subcommand)]
#[clap(rename_all = "snake_case")]
pub enum SetCommand {
    /// Enable v2 storage layout (static files + RocksDB routing)
    StorageV2 {
        #[clap(action(ArgAction::Set))]
        value: bool,
    },
}

impl Command {
    /// Execute the command
    pub fn execute<N: ProviderNodeTypes>(self, tool: &DbTool<N>) -> eyre::Result<()> {
        match self.command {
            Subcommands::Get => self.get(tool),
            Subcommands::Set(cmd) => self.set(cmd, tool),
        }
    }

    fn get<N: ProviderNodeTypes>(&self, tool: &DbTool<N>) -> eyre::Result<()> {
        let provider = tool.provider_factory.provider()?;
        let storage_settings = provider.storage_settings()?;

        match storage_settings {
            Some(settings) => {
                println!("Current storage settings:");
                println!("{settings:#?}");
            }
            None => {
                println!("No storage settings found.");
            }
        }

        Ok(())
    }

    fn set<N: ProviderNodeTypes>(&self, cmd: SetCommand, tool: &DbTool<N>) -> eyre::Result<()> {
        let provider_rw = tool.provider_factory.database_provider_rw()?;
        let settings = provider_rw.storage_settings()?;
        if settings.is_none() {
            println!("No storage settings found, creating new settings.");
        }

        let mut settings = settings.unwrap_or_else(StorageSettings::v1);

        match cmd {
            SetCommand::StorageV2 { value } => {
                if settings.storage_v2 == value {
                    println!("storage_v2 is already set to {value}");
                    return Ok(());
                }
                settings.storage_v2 = value;
                println!("Set storage_v2 = {value}");
            }
        }

        provider_rw.write_storage_settings(settings)?;
        provider_rw.commit()?;

        println!("Storage settings updated successfully.");

        Ok(())
    }

    /// Reads storage settings directly from MDBX metadata (skips static file initialization).
    ///
    /// Used when legacy static file jars prevent `Environment::init` from opening storage.
    pub fn execute_get_db_only(db_path: &Path, database_args: &DatabaseArgs) -> eyre::Result<()> {
        let db = open_db_read_only(db_path, database_args.database_args())?;
        let settings_bytes =
            Database::view(&db, |tx| tx.get::<Metadata>(STORAGE_SETTINGS.to_string()))?
                .map_err(|err| eyre::eyre!(err))?;

        match settings_bytes {
            Some(bytes) => {
                let settings = serde_json::from_slice::<StorageSettings>(&bytes)
                    .map_err(|err| eyre::eyre!("invalid storage_settings metadata: {err}"))?;
                println!("Current storage settings:");
                println!("{settings:#?}");
            }
            None => {
                println!("No storage settings found.");
            }
        }

        Ok(())
    }
}
