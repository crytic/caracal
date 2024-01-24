use array::ArrayTrait;

#[starknet::contract]
mod UnusedArguments {
    #[storage]
    struct Storage {
        value: felt252,
    }

    #[external(v0)]
    fn unused_1(ref self: ContractState, a: felt252, b: felt252) {
        self.value.write(a);
    }

    #[external(v0)]
    fn unused_2(self: @ContractState, array: Array::<felt252>, l: felt252) -> felt252{
        let _a = 1; // Need this otherwise the function is optimized away and put directly in the wrapper
        1
    }

}
