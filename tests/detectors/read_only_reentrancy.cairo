#[abi]
trait IAnotherContract {
    fn foo(a: felt252);
}

#[contract]
mod TestContract {
    use super::IAnotherContractDispatcherTrait;
    use super::IAnotherContractDispatcher;
    use starknet::ContractAddress;
    
    struct Storage {
        a: felt252,
        b: felt252,
    }

    #[view]
    fn get_a() -> felt252 {
        a::read()
    }

    #[view]
    fn get_b() -> felt252 {
        b::read()
    }

    #[external]
    fn bad(address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.foo(4);
        a::write(4);
    }

    #[external]
    fn ok(address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.foo(4);
    }

}
