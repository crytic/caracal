#[starknet::contract]
mod UnenforcedView {
    #[storage]
    struct Storage {
        value: felt252,
    }

    #[external(v0)]
    fn writes_to_storage_indirect(self: @ContractState, val: felt252) {
       f1(val);
    }

    fn f1(val: felt252) {
        f2(val);
    }


    fn f2(val: felt252) {
        value::write(val);
    }

    #[external(v0)]
    fn writes_to_storage_direct(self: @ContractState, val:felt252) {
        value::write(val);
    }

    #[external(v0)]
    fn recursive_storage_write_direct(self: @ContractState, val: felt252) {
        if val == 0 {
            ()
        }
        value::write(val);
        recursive_storage_write_direct(val-1);
    }

    #[external(v0)]
    fn recursive_storage_write_indirect(self: @ContractState, val: felt252) {
        if val == 0 {
            ()
        }
        f3(val);
    }

    fn f3(val: felt252) {
        value::write(val);
        recursive_storage_write_indirect(val-1);
    }


    #[external(v0)]
    fn does_not_write_to_storage(self: @ContractState) -> felt252 {
        value::read()
    }
}



