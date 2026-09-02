//! `reth db settings` command for managing storage settings

use clap::{ArgAction, Parser, Subcommand};
use reth_db::open_db_read_only;
use reth_db_api::{database::Database, tables::Metadata, transaction::DbTx};
use reth_db_common::DbTool;
use reth_node_core::args::DatabaseArgs;
use reth_provider::{
    providers::ProviderNodeTypes, DBProvider, DatabaseProviderFactory, MetadataProvider,
    StorageSettings,
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
        match cmd {
            SetCommand::StorageV2 { value } => {
                let current = tool
                    .provider_factory
                    .provider()?
                    .storage_settings()?
                    .unwrap_or_else(StorageSettings::v1);

                if current.storage_v2 == value {
                    println!("storage_v2 is already set to {value}");
                    return Ok(());
                }

                eyre::bail!(
                    "refusing to change storage_v2 directly: this changes the on-disk routing \
                     between MDBX, static files, and RocksDB. Use `reth db migrate-v2` to \
                     migrate from v1 to v2. Downgrading a v2 database is unsupported."
                );
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use reth_provider::{
        test_utils::create_test_provider_factory, DatabaseProviderFactory, MetadataProvider,
        MetadataWriter,
    };

    #[test]
    fn rejects_direct_storage_layout_change() {
        let provider_factory = create_test_provider_factory();
        let tool = DbTool::new(provider_factory.clone()).expect("db tool");

        let err = Command { command: Subcommands::Set(SetCommand::StorageV2 { value: true }) }
            .execute(&tool)
            .expect_err("must reject an unmigrated v1 database");

        assert!(err.to_string().contains("reth db migrate-v2"));
        assert_eq!(
            provider_factory
                .provider()
                .expect("provider")
                .storage_settings()
                .expect("storage settings"),
            None
        );
    }

    #[test]
    fn accepts_noop_storage_layout_change() {
        let provider_factory = create_test_provider_factory();
        let tool = DbTool::new(provider_factory.clone()).expect("db tool");

        {
            let provider_rw = provider_factory.database_provider_rw().expect("rw provider");
            provider_rw
                .write_storage_settings(StorageSettings::v2())
                .expect("write storage settings");
            provider_rw.commit().expect("commit storage settings");
        }

        Command { command: Subcommands::Set(SetCommand::StorageV2 { value: true }) }
            .execute(&tool)
            .expect("same layout is a no-op");
    }
}
