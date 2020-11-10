#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod factory {
    use erc20::Erc20;
    use ink_storage::{
        traits::{PackedLayout, SpreadLayout},
        collections::HashMap as StorageHashMap,
    };

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        CreateError,
        UnknownHash,
    }

    #[derive(
        scale::Encode,
        scale::Decode,
        SpreadLayout,
        PackedLayout,
        PartialOrd,
        PartialEq,
        Clone,
        Debug,
        Ord,
        Eq,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractType {
        Erc20,
        Erc721,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct Factory {
        instances: StorageHashMap<AccountId, Erc20>,
        contract_hash: StorageHashMap<ContractType, Hash>,
    }

    impl Factory {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                instances: StorageHashMap::new(),
                contract_hash: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn code_hash_of(&self, contract_type: ContractType) -> Result<Hash> {
            self.contract_hash.get(&contract_type).copied().ok_or(Error::UnknownHash)
        }

        #[ink(message)]
        pub fn add_code_hash(&mut self, contract_type: ContractType, code_hash: Hash) -> Result<()> {
            self.contract_hash.insert(contract_type, code_hash);
            Ok(())
        }

        #[ink(message, payable)]
        pub fn create_contract(
            &mut self,
            init_value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let endowment = self.env().transferred_balance();
            let erc20 = match self.create_erc20(init_value, endowment) {
                Ok(instance)  => instance,
                Err(e) => return Err(e),
            };
            self.instances.insert(caller, erc20);
            Ok(())
        }

        fn create_erc20(
            &self,
            init_value: Balance,
            endowment: Balance,
        ) -> Result<Erc20> {
            let erc20_code_hash = match self.code_hash_of(ContractType::Erc20) {
                Ok(code_hash)  => code_hash,
                Err(e) => return Err(e),
            };
            Ok(Erc20::new(init_value)
                .endowment(endowment)
                .code_hash(erc20_code_hash)
                .instantiate()
                .expect("failed at instantiating the `Erc20` contract"))
        }
    }
}
