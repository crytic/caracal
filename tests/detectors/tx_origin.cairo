#[starknet::interface]
trait ITxOrigin<T> {
    fn bad(self: @T) -> bool;
    fn bad_indirect(self: @T) -> bool;
    fn good(self: @T) -> starknet::ContractAddress;
}

#[starknet::contract]
mod TxOrigin {
    use core::box::BoxTrait;
    use core::result::ResultTrait;
    use starknet::{ ContractAddress, contract_address_const, get_tx_info };

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl TxOrigin of super::ITxOrigin<ContractState> {
        fn bad(self: @ContractState) -> bool {
            let tx_info = get_tx_info().unbox();
            let tx_origin = tx_info.account_contract_address;
            let owner = contract_address_const::<1>();
            tx_origin == owner
        }

        fn bad_indirect(self: @ContractState) -> bool {
            let tx_info = get_tx_info().unbox();
            let tx_origin = tx_info.account_contract_address;
            self._check_tx_origin(tx_origin)
        }

        fn good(self: @ContractState) -> ContractAddress {
            let tx_info = get_tx_info().unbox();
            tx_info.account_contract_address
        }
    }

    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {
        fn _check_tx_origin(self: @ContractState, tx_origin: ContractAddress) -> bool {
            let owner = contract_address_const::<1>();
            tx_origin == owner
        }
    }
}