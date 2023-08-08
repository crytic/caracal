use option::OptionTrait;

#[starknet::contract]
mod UnusedReturn {
    #[storage]
    struct Storage {
        value: felt252,
    }

    #[external(v0)]
    fn unused_return_1(ref self: ContractState, amount: felt252) {
        f_1(ref self, amount);
    }

    #[external(v0)]
    fn unused_return_2(ref self: ContractState, amount: felt252) -> felt252 {
        let (a,b,d) = f_2(amount);
        let c = a  ;
        a
    }

    #[external(v0)]
    fn unused_return_3(ref self: ContractState, amount: felt252) {
        f_3(amount);
    }

    #[external(v0)]
    fn unused_return_4(ref self: ContractState, amount: felt252) {
        f_4(amount);
    }

    #[external(v0)]
    fn unused_return_5(ref self: ContractState) {
        let a = f_5(ref self);        
    }

    #[external(v0)]
    fn no_report(ref self: ContractState) {
        let a = self.value.read();
    }

    #[external(v0)]
    fn no_report2(ref self: ContractState) -> felt252 {
        let a = f_5(ref self);
        a
    }

    #[external(v0)]
    fn no_report3(ref self: ContractState) -> felt252 {
        f_5(ref self)
    }

    fn f_1(ref self: ContractState, amount: felt252) -> felt252 {
        self.value.write(amount);
        23
    }

    fn f_2(amount: felt252) -> (felt252, felt252, felt252) {
        (amount, amount, amount)
    }

    fn f_3(amount: felt252) -> felt252 {
        amount
    }

    fn f_4(amount: felt252) -> Option::<felt252> {
        Option::Some(amount)
    }

    fn f_5(ref self: ContractState) -> felt252 {
        let a = self.value.read();
        a * 2
    }

}
