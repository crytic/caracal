#[starknet::contract]
mod TxOrigin {
    use core::box::BoxTrait;
    use core::result::ResultTrait;
    use starknet::{ ContractAddress, contract_address_const, get_tx_info };

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn bad(self: @ContractState) -> bool {
        let tx_info = get_tx_info().unbox();
        let tx_origin = tx_info.account_contract_address;
        let owner = contract_address_const::<1>();
        tx_origin == owner
    }

    #[external(v0)]
    fn bad_indirect(self: @ContractState) -> bool {
        let tx_info = get_tx_info().unbox();
        let tx_origin = tx_info.account_contract_address;
        _check_tx_origin(tx_origin)
    }

    #[external(v0)]
    fn good(self: @ContractState) -> ContractAddress {
        let tx_info = get_tx_info().unbox();
        tx_info.account_contract_address
    }


    fn _check_tx_origin(tx_origin: ContractAddress) -> bool {
        let owner = contract_address_const::<1>();
        tx_origin == owner
    }
    
}