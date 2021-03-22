#[rpc]
pub trait LockdropApi<BlockHash> {
    #[rpc(name = "Lockdrop_deployContract")]
    fn deploy_contract(
        &self,
        at: Option<BlockHash>
    ) -> Result<()>;
}

/// A struct that implements the `LockdropApi`.
pub struct Lockdrop<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Lockdrop<C, M> {
    /// Create new `Lockdrop` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

impl<C, Block> SumStorageApi<<Block as BlockT>::Hash>
    for SumStorage<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi,
    C: HeaderBackend<Block>,
    C::Api: SumStorageRuntimeApi<Block>,
{
    fn deploy_contract(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        ));

        let runtime_api_result = api.deploy_contract(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}