#[contract]
mod UncheckedL1HandlerFrom {

    // Required to make consider the cairo file as a contract
    #[external]
    fn dummy() {
        1;
    }

    #[l1_handler]
    fn bad(from_address: felt252) {
        from_address + 1;
    }

    #[l1_handler]
    fn good(from_address: felt252) {
        assert(from_address == 0, 'Wrong L1 sender');
    }

}