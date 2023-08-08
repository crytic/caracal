#[starknet::interface]
trait IAnotherContract<T> {
    fn foo(self: @T, a: felt252);
}

#[starknet::contract]
mod TestContract {
    use super::IAnotherContractDispatcherTrait;
    use super::IAnotherContractDispatcher;
    use starknet::ContractAddress;
    
    #[storage]
    struct Storage {
        a: felt252,
        b: felt252,
    }

    #[external(v0)]
    fn get_a(self: @ContractState) -> felt252 {
        self.a.read()
    }

    #[external(v0)]
    fn get_b(self: @ContractState) -> felt252 {
        self.b.read()
    }

    #[external(v0)]
    fn bad(ref self: ContractState, address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.foo(4);
        self.a.write(4);
    }

    #[external(v0)]
    fn ok(ref self: ContractState, address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.foo(4);
    }

}
