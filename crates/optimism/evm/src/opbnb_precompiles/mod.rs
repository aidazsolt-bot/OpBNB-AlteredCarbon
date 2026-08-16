//! opBNB Fermat/Haber precompile overlays.
//!
//! Historically these lived in patched bnb-chain/revm (`Precompiles::fermat` / `haber`),
//! selected via the local SpecId ladder in `config.rs`. Stock `op-revm` has no Fermat/Haber
//! SpecIds, so the same tables are injected on top of [`OpPrecompiles`] when the chainspec
//! says the forks are active.

mod bls;
mod cometbft;
mod dedup;
mod error;

use alloc::borrow::Cow;
use op_revm::{precompiles::OpPrecompiles, OpSpecId};
use reth_evm::precompiles::PrecompilesMap;
use revm::precompile::{secp256r1, Precompiles};

/// Flags selecting the historical Fermat/Haber precompile overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpBnbPrecompileFlags {
    /// Inject Fermat precompiles at `0x66` / `0x67`.
    pub fermat: bool,
    /// Inject Haber early P256 at `0x0100` when Fjord has not yet activated.
    pub haber_p256: bool,
}

impl OpBnbPrecompileFlags {
    /// Returns `true` when no overlay is needed (stock [`OpPrecompiles`] is sufficient).
    pub const fn is_empty(self) -> bool {
        !self.fermat && !self.haber_p256
    }
}

/// Builds the precompile map for `spec` with optional Fermat/Haber overlays.
pub fn opbnb_precompiles(spec: OpSpecId, flags: OpBnbPrecompileFlags) -> PrecompilesMap {
    if flags.is_empty() {
        return PrecompilesMap::from_static(OpPrecompiles::new_with_spec(spec).precompiles());
    }
    PrecompilesMap::new(Cow::Owned(build_overlay(spec, flags)))
}

fn build_overlay(spec: OpSpecId, flags: OpBnbPrecompileFlags) -> Precompiles {
    let mut precompiles = OpPrecompiles::new_with_spec(spec).precompiles().clone();
    if flags.fermat {
        // op-geth `cometBFTLightBlockValidate` @ 0x67 always returns the pre-update
        // `validatorSetChanged` (BSC Hertz semantics). There is no opBNB "before Hertz"
        // map — injecting BEFORE_HERTZ forced `false` and skipped IBC validator-set
        // SSTOREs → under-gas receipts (FLOW-X04 / PIPE-014 @ 21591154 tx#10).
        precompiles.extend([
            bls::BLS_SIGNATURE_VALIDATION,
            cometbft::COMETBFT_LIGHT_BLOCK_VALIDATION,
        ]);
    }
    if flags.haber_p256 {
        precompiles.extend([secp256r1::P256VERIFY]);
    }
    precompiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::precompile::u64_to_address;

    #[test]
    fn fermat_injects_bls_and_cometbft() {
        let flags = OpBnbPrecompileFlags { fermat: true, haber_p256: false };
        let overlaid = build_overlay(OpSpecId::BEDROCK, flags);
        assert!(overlaid.contains(&u64_to_address(102)));
        assert!(overlaid.contains(&u64_to_address(103)));
        assert!(!OpPrecompiles::new_with_spec(OpSpecId::BEDROCK)
            .precompiles()
            .contains(&u64_to_address(102)));
    }

    #[test]
    fn haber_injects_p256() {
        let flags = OpBnbPrecompileFlags { fermat: true, haber_p256: true };
        let overlaid = build_overlay(OpSpecId::ECOTONE, flags);
        assert!(overlaid.contains(&u64_to_address(0x100)));
        assert!(overlaid.contains(&u64_to_address(102)));
    }
}
