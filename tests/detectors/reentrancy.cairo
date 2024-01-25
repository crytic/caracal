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
    
    #[storage]
    struct Storage {
        a: felt252,
        b: felt252,
    }

    #[external(v0)]
    fn good1(ref self: ContractState, address: ContractAddress) {
        let a = self.a.read();
        self.a.write(4);
        IAnotherContractDispatcher { contract_address: address }.foo(a);
    }

    #[external(v0)]
    fn good2(ref self: ContractState, address: ContractAddress) {
        let a = self.a.read();
        IAnotherContractDispatcher { contract_address: address }.safe_foo(a);
        self.a.write(4);
    }

    #[external(v0)]
    fn bad1(ref self: ContractState, address: ContractAddress) {
        let a = self.a.read();
        IAnotherContractDispatcher { contract_address: address }.foo(a);
        self.a.write(4);
    }
    
    #[external(v0)]
    fn bad2(ref self: ContractState, address: ContractAddress) {
        if 2 == 2 {
            let a = self.a.read();
            IAnotherContractDispatcher { contract_address: address }.foo(a);
        } else {
            let b = self.b.read();
            IAnotherContractDispatcher { contract_address: address }.foo(b);
        }
        self.a.write(4);
        self.b.write(4);
    }

    #[external(v0)]
    fn bad3(ref self: ContractState, address: ContractAddress) {
        let _a = self.a.read();
        internal_ext_call(address);
        self.a.write(4);
    }

    fn internal_ext_call(address: ContractAddress) {
        IAnotherContractDispatcher { contract_address: address }.foo(4);
    }

    #[external(v0)]
    fn bad4(ref self: ContractState, address: ContractAddress) {
        internal_ext_call2(ref self, address);
        self.a.write(4);
    }

    #[external(v0)]
    fn bad5(ref self: ContractState, address: ContractAddress) {
        let a = self.a.read();
        loop {
            IAnotherContractDispatcher { contract_address: address }.foo(a);
            break;
        };
        self.a.write(4);
    }

    fn internal_ext_call2(ref self: ContractState, address: ContractAddress) {
        let _a = self.a.read();
        IAnotherContractDispatcher { contract_address: address }.foo(4);
    }

}
