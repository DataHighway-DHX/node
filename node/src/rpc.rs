use std::{
    fmt,
    sync::Arc,
};

use datahighway_runtime::{
    opaque::Block,
    AccountId,
    Balance,
    Index,
};
use sc_consensus_babe::{
    Config,
    Epoch,
};

use sc_consensus_epochs::SharedEpochChanges;
use sc_keystore::KeyStorePtr;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{
    Error as BlockChainError,
    HeaderBackend,
    HeaderMetadata,
};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_transaction_pool::TransactionPool;
use substrate_frame_rpc_system::AccountNonceApi;

pub use sc_rpc_api::DenyUnsafe;

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: Config,
    /// BABE pending epoch changes.
    pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: KeyStorePtr,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, M, SC>(deps: FullDeps<C, P, SC>) -> jsonrpc_core::IoHandler<M>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BabeApi<Block>,
    // TODO
    // C::Api: orml_oracle_rpc::OracleRuntimeApi<Block, CurrencyId, TimeStampedPrice>,
    <C::Api as sp_api::ApiErrorExt>::Error: fmt::Debug,
    P: TransactionPool + 'static,
    M: jsonrpc_core::Metadata + Default,
    SC: SelectChain<Block> + 'static,
{
    // TODO
    // use orml_oracle_rpc::{Oracle, OracleApi};
    use pallet_transaction_payment_rpc::{
        TransactionPayment,
        TransactionPaymentApi,
    };

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool: _,
        select_chain: _,
        babe,
        deny_unsafe: _,
    } = deps;
    let BabeDeps {
        keystore: _,
        babe_config: _,
        shared_epoch_changes: _,
    } = babe;

    // io.extend_with(SystemApi::to_delegate(FullSystem::new(client.clone(), pool, deny_unsafe))); TODO#ILYA
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone())));
    // io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(BabeRpcHandler::new(
    //     client.clone(),
    //     shared_epoch_changes,
    //     keystore,
    //     babe_config,
    //     select_chain,
    //     deny_unsafe,
    // )));
    // TODO: Add Oracle
    // io.extend_with(OracleApi::to_delegate(Oracle::new(client))); TODO#ILYA

    io
}
