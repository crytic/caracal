use debug::PrintTrait;
#[starknet::contract]
mod Felt252Overflow {
    #[storage]
    struct Storage {
        a: felt252,
        b: felt252
    }

    #[external(v0)]
    fn bad(ref self:ContractState) -> felt252 {
        let max: felt252 = 0x800000000000011000000000000000000000000000000000000000000000000;
        self.a.write(max +1);
        return self.a.read();
    }

    #[external(v0)]
    fn bad_user_controlled(ref self:ContractState, user_param:felt252) {
        let a = self.a.read();
        self.b.write(a + user_param);
    }

    #[external(v0)]
    fn bad_sub(ref self:ContractState)  {
        let min: felt252 =0;
        self.a.write(min-1);
    }

    #[external(v0)]
    fn bad_sub_user_controlled(ref self:ContractState, user_param:felt252) {
        let a = self.a.read();
        self.b.write(a - user_param);
    }

    // #[external(v0)]
    // fn bad_test_div(self:@ContractState) {
    //     let div:felt252 = 7/3;
    //     div.print();

    // }

}