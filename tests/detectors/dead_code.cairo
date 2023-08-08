#[starknet::contract]
mod DeadCode {

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn use_add_1(self: @ContractState, amount: felt252) -> felt252{
        add_1(amount)
    }

    fn add_1(amount: felt252) -> felt252 {
        amount + 1
    }

    fn add_2(amount: felt252) -> felt252 {
        amount + 2
    }

}
