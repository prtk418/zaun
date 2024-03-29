use async_trait::async_trait;

use crate::Error;

use alloy::{
    primitives::{Address, Bytes, U256, I256},
    network::Ethereum,
    providers::Provider,
    rpc::types::eth::TransactionReceipt,
    sol,
    sol_types::ContractError,
};

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface ProxySupport {
        function isFrozen() external view virtual returns (bool);
        function initialize(bytes calldata data) external notCalledDirectly;
    }
);

#[async_trait]
pub trait ProxySupportTrait<P: Provider<Ethereum>> {
    async fn is_frozen(&self) -> Result<bool, Error<P>>;
    async fn initialize(&self, data: Bytes) -> Result<Option<TransactionReceipt>, Error<P>>;
    async fn initialize_with<const N: usize>(
        &self,
        data: ProxyInitializeData<N>,
    ) -> Result<Option<TransactionReceipt>, Error<P>>;
}

#[async_trait]
impl<T, P: Provider<Ethereum>> ProxySupportTrait<P> for T
where
    T: AsRef<ProxySupport::ProxySupportInstance<Ethereum, T, P>> + Send + Sync,
{
    async fn is_frozen(&self) -> Result<bool, Error<P>> {
        self.as_ref().is_frozen().call().await.map_err(Into::into)
    }

    async fn initialize(&self, data: Bytes) -> Result<Option<TransactionReceipt>, Error<P>> {
        self.as_ref()
            .initialize(data)
            .send()
            .await
            .map_err(Into::<ContractError<P>>::into)?
            .await
            .map_err(Into::into)
    }

    async fn initialize_with<const N: usize>(
        &self,
        data: ProxyInitializeData<N>,
    ) -> Result<Option<TransactionReceipt>, Error<P>> {
        self.initialize(data.into()).await
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CoreContractState {
    pub state_root: U256,
    pub block_number: I256,
    pub block_hash: U256,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CoreContractInitData {
    pub program_hash: U256,
    pub verifier_address: Address,
    pub config_hash: U256,
    pub initial_state: CoreContractState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProxyInitializeData<const N: usize> {
    pub sub_contract_addresses: [Address; N],
    pub eic_address: Address,
    pub init_data: CoreContractInitData,
}

impl<const N: usize> Into<Vec<u8>> for ProxyInitializeData<N> {
    fn into(self) -> Vec<u8> {
        [
            self.sub_contract_addresses.encode(),
            self.eic_address.encode(),
            self.init_data.encode(),
        ]
        .concat()
    }
}

impl<const N: usize> Into<Bytes> for ProxyInitializeData<N> {
    fn into(self) -> Bytes {
        Into::<Vec<u8>>::into(self).into()
    }
}

#[cfg(test)]
mod tests {
    use super::ProxyInitializeData;

    #[test]
    fn test_initialize_calldata_encoding() {
        let calldata = ProxyInitializeData::<0> {
            sub_contract_addresses: [],
            eic_address: Default::default(),
            init_data: Default::default(),
        };
        let bytes: Vec<u8> = calldata.into();
        assert_eq!(bytes, [0u8; 7 * 32].to_vec());
    }
}
