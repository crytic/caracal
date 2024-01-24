use option::OptionTrait;
use serde::Serde;
#[starknet::contract]
mod UnusedReturn {
    #[storage]
    struct Storage {
        value: felt252,
    }
    #[derive(Copy,Drop,Serde)]
    struct TestStruct {
        val1: felt252,
        val2: felt252,
        val3: felt252
    }

    #[external(v0)]
    fn unused_return_1(ref self: ContractState, amount: felt252) {
        f_1(ref self, amount);
    }

    #[external(v0)]
    fn unused_return_2(ref self: ContractState, amount: felt252) -> felt252 {
        let (a,_b,_d) = f_2(amount);
        let _c = a;
        a
    }

    #[external(v0)]
    fn unused_return_3(ref self: ContractState, amount: felt252) {
        f_3(amount);
    }

    #[external(v0)]
    fn unused_return_4(ref self: ContractState, amount: felt252) {
        let _a = f_4(amount);
    }

    #[external(v0)]
    fn unused_return_5(ref self: ContractState) {
        let a = f_5(ref self);
    }

    #[external(v0)]
    fn unused_return_6(ref self: ContractState, s: TestStruct) {
        let a = f_6(ref self, s.val1,s.val2);
    }

    #[external(v0)]
    fn unused_return_7(ref self: ContractState, s: TestStruct) {
        let a = f_7(ref self, s);
    }

    #[external(v0)]
    fn no_report(ref self: ContractState) {
        let _a = self.value.read();
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

    #[external(v0)]
    fn no_report4(ref self: ContractState, s: TestStruct) -> felt252 {
        let a = f_6(ref self, s.val1,s.val2);
        s.val3 + a
    }

    #[external(v0)]
    fn no_report5(ref self: ContractState, s: TestStruct) -> felt252 {
        let a = f_6(ref self, s.val1,s.val2);
        a
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

    fn f_6(ref self:ContractState, amount1: felt252, amount2: felt252) -> felt252 {
        let a = self.value.read();
        let ret = amount1 * amount2;
        ret + a
    }

    fn f_7(ref self:ContractState, s:TestStruct) -> felt252 {
        let a = self.value.read();
        s.val1 * s.val2 * s.val3 + a
    }

}