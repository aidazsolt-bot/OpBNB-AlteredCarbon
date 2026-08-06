//! API of a signed transaction.

use alloc::fmt;
use core::hash::Hash;

use crate::InMemorySize;
use alloy_consensus::Transaction;
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::{keccak256, Address, Signature, TxHash, B256};
use revm::context::TxEnv;

/// Opaque error type for sender recovery, re-exported from `alloy-consensus`.
pub use alloy_consensus::crypto::RecoveryError;

/// A signed transaction.
pub trait SignedTransaction: 'static +
    fmt::Debug
    + Clone
    + PartialEq
    + Eq
    + Hash
    + Send
    + Sync
    + Unpin
    + InMemorySize
    + serde::Serialize
    + for<'a> serde::Deserialize<'a>
    + alloy_rlp::Encodable
    + alloy_rlp::Decodable
    + Encodable2718
    + Decodable2718
{
    /// Transaction type that is signed.
    type Transaction: Transaction;

    /// Returns reference to transaction hash.
    fn tx_hash(&self) -> &TxHash;

    /// Returns reference to transaction.
    fn transaction(&self) -> &Self::Transaction;

    /// Returns reference to signature.
    fn signature(&self) -> &Signature;

    /// Recover signer from signature and hash.
    ///
    /// Returns `None` if the transaction's signature is invalid following [EIP-2](https://eips.ethereum.org/EIPS/eip-2), see also `reth_primitives::transaction::recover_signer`.
    ///
    /// Note:
    ///
    /// This can fail for some early ethereum mainnet transactions pre EIP-2, use
    /// [`Self::recover_signer_unchecked`] if you want to recover the signer without ensuring that
    /// the signature has a low `s` value.
    fn recover_signer(&self) -> Option<Address>;

    /// Recover signer from signature and hash _without ensuring that the signature has a low `s`
    /// value_.
    ///
    /// Returns `None` if the transaction's signature is invalid, see also
    /// `reth_primitives::transaction::recover_signer_unchecked`.
    fn recover_signer_unchecked(&self) -> Option<Address>;

    /// Create a new signed transaction from a transaction and its signature.
    ///
    /// This will also calculate the transaction hash using its encoding.
    fn from_transaction_and_signature(transaction: Self::Transaction, signature: Signature)
        -> Self;

    /// Calculate transaction hash, eip2728 transaction does not contain rlp header and start with
    /// tx type.
    fn recalculate_hash(&self) -> B256 {
        keccak256(self.encoded_2718())
    }

    /// Fills [`TxEnv`] with an [`Address`] and transaction.
    fn fill_tx_env(&self, tx_env: &mut TxEnv, sender: Address)
    where
        for<'a> TxEnv: alloy_evm::FromRecoveredTx<&'a Self>,
    {
        *tx_env = <TxEnv as alloy_evm::FromRecoveredTx<&Self>>::from_recovered_tx(&self, sender);
    }

    /// Returns a [`super::Recovered`] with the given sender, without additional validation.
    fn with_signer(self, signer: Address) -> super::Recovered<Self>
    where
        Self: Sized,
    {
        super::Recovered::new_unchecked(self, signer)
    }
}

impl<T> SignedTransaction for alloy_consensus::EthereumTxEnvelope<T>
where
    T: alloy_consensus::transaction::RlpEcdsaEncodableTx
        + alloy_consensus::transaction::RlpEcdsaDecodableTx
        + alloy_consensus::SignableTransaction<Signature>
        + Unpin
        + Clone
        + PartialEq
        + Eq
        + core::hash::Hash
        + fmt::Debug
        + Send
        + Sync
        + InMemorySize
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + 'static,
    Self: Transaction,
{
    type Transaction = Self;

    fn tx_hash(&self) -> &TxHash {
        self.hash()
    }

    fn transaction(&self) -> &Self::Transaction {
        self
    }

    fn signature(&self) -> &Signature {
        Self::signature(self)
    }

    fn recover_signer(&self) -> Option<Address> {
        <Self as alloy_consensus::transaction::SignerRecoverable>::recover_signer(self).ok()
    }

    fn recover_signer_unchecked(&self) -> Option<Address> {
        <Self as alloy_consensus::transaction::SignerRecoverable>::recover_signer_unchecked(self)
            .ok()
    }

    fn from_transaction_and_signature(transaction: Self::Transaction, signature: Signature) -> Self {
        let _ = signature;
        transaction
    }
}
