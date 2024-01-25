#[starknet::interface]
trait IAnotherContract<T> {
    fn foo(self: @T, a: felt252);
    fn safe_foo(self: @T, a: felt252);
}

#[starknet::contract]
mod TestContract {
    use super::IAnotherContractDispatcherTrait;
    use super::IAnotherContractDispatcher;
    use starknet::ContractAddress;

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        MyEvent: MyEvent,
    }
    
    #[derive(Drop, starknet::Event)]
    struct MyEvent {}

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn good1(ref self: ContractState, address: ContractAddress) {
        self.emit(MyEvent { });
        IAnotherContractDispatcher { contract_address: address }.foo(4);
    }

    #[external(v0)]
    fn good2(ref self: ContractState, address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.safe_foo(4);
        self.emit(MyEvent { });
    }

    #[external(v0)]
    fn bad1(ref self: ContractState, address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.foo(4);
        self.emit(MyEvent { });
    }

}
