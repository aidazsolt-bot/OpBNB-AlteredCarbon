//! opBNB `PreContractForkBlock` irregular state transition.
//!
//! Matches bnb-chain/op-geth `consensus/misc.ApplyPreContractHardFork`: rename WBNB
//! name/symbol storage and selfdestruct the governance token predeploy. Applied once at the
//! fork block, before transactions (same timing as op-geth `StateProcessor`).

use alloy_evm::Database;
use alloy_primitives::{Address, U256, address, uint};
use revm::{
    DatabaseCommit,
    primitives::HashMap,
    state::{Account, EvmStorageSlot, TransactionId},
};

/// WBNB predeploy (`0x4200…0006`).
const WBNB_CONTRACT: Address = address!("0x4200000000000000000000000000000000000006");
/// Governance token predeploy (`0x4200…0042`).
const GOVERNANCE_TOKEN: Address = address!("0x4200000000000000000000000000000000000042");

const NAME_SLOT: U256 = U256::ZERO;
const SYMBOL_SLOT: U256 = uint!(1_U256);
/// ERC-20 name slot value for `"Wrapped BNB"`.
const NAME_VALUE: U256 =
    uint!(0x5772617070656420424e42000000000000000000000000000000000000000016_U256);
/// ERC-20 symbol slot value for `"WBNB"`.
const SYMBOL_VALUE: U256 =
    uint!(0x57424e4200000000000000000000000000000000000000000000000000000008_U256);

/// Apply the PreContractForkBlock state mutation (caller decides fork activation).
pub(crate) fn apply_pre_contract_hardfork<DB>(db: &mut DB) -> Result<(), DB::Error>
where
    DB: Database + DatabaseCommit,
{
    let mut wbnb: Account = db.basic(WBNB_CONTRACT)?.unwrap_or_default().into();
    let name_original = db.storage(WBNB_CONTRACT, NAME_SLOT)?;
    let symbol_original = db.storage(WBNB_CONTRACT, SYMBOL_SLOT)?;
    wbnb.storage.insert(
        NAME_SLOT,
        EvmStorageSlot::new_changed(name_original, NAME_VALUE, TransactionId::ZERO),
    );
    wbnb.storage.insert(
        SYMBOL_SLOT,
        EvmStorageSlot::new_changed(symbol_original, SYMBOL_VALUE, TransactionId::ZERO),
    );
    wbnb.mark_touch();

    let mut governance: Account = db.basic(GOVERNANCE_TOKEN)?.unwrap_or_default().into();
    governance.selfdestruct();

    db.commit(HashMap::from_iter([(WBNB_CONTRACT, wbnb), (GOVERNANCE_TOKEN, governance)]));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::{
        Database,
        database::{CacheDB, EmptyDB},
    };

    #[test]
    fn apply_sets_wbnb_slots_and_selfdestructs_governance() {
        let mut db = CacheDB::new(EmptyDB::default());
        apply_pre_contract_hardfork(&mut db).unwrap();

        assert_eq!(Database::storage(&mut db, WBNB_CONTRACT, NAME_SLOT).unwrap(), NAME_VALUE);
        assert_eq!(Database::storage(&mut db, WBNB_CONTRACT, SYMBOL_SLOT).unwrap(), SYMBOL_VALUE);

        let gov = db.cache.accounts.get(&GOVERNANCE_TOKEN).expect("gov committed");
        assert!(gov.info.is_empty(), "governance token should be cleared after selfdestruct");
    }
}
