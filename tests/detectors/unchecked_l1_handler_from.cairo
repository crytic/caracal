#[starknet::contract]
mod UncheckedL1HandlerFrom {
    #[storage]
    struct Storage {}

    #[l1_handler]
    fn bad(self: @ContractState, from_address: felt252) {
        from_address + 1;
    }

    #[l1_handler]
    fn good(self: @ContractState, from_address: felt252) {
        assert(from_address == 0, 'Wrong L1 sender');
    }

    #[l1_handler]
    fn good2(self: @ContractState, from_address: felt252) {
        check_from_address(from_address);
    }

    // Check from address in a private function
    fn check_from_address(from_address: felt252) {
        assert(from_address != 0, 'Wrong L1 sender');
    }

    #[l1_handler]
    fn good3(self: @ContractState, from_address: felt252) {
        let x = check_recursive(from_address, 0);
        x + 2;
    }

    // Test recursive or looped private function calls
    fn check_recursive(from_address: felt252, number: felt252) -> felt252 {
        if (number == 2) {
            return number;
        }
        assert(from_address != 0, 'Wrong L1 sender');
        return check_recursive(from_address, number + 1);
    }
}
