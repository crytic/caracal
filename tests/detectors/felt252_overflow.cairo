use debug::PrintTrait;
#[starknet::contract]
mod Felt252Overflow {
    #[storage]
    struct Storage {
        a: felt252,
        b: felt252
    }

    #[external(v0)]
    fn bad_mul_controlled(ref self:ContractState,param1:felt252, param2:felt252, param3:felt252) -> felt252 {
        let max: felt252 = param1 * param2;
        let my: felt252 = param1 * param1;
        self.b.write(max * param3);
        return self.a.read();
    }

    #[external(v0)]
    fn bad_add_controlled(ref self:ContractState, user_param:felt252, user_param2: felt252) {
        let a = self.a.read();
        let controlled = bad_add(user_param, user_param2);
        self.b.write(a + user_param+ user_param2);
    }

    #[external(v0)]
    fn bad_sub_uncontrolled(ref self:ContractState)  {
        let min: felt252 =0;
        self.a.write(min-1);
    }

    #[external(v0)]
    fn bad_add_uncontrolled(self:@ContractState) -> felt252 {
        let max: felt252 = 0 - 1;
        bad_add(max,2)
    }

    fn bad_add(param1:felt252, param2:felt252) -> felt252{
        param1 + param2
    }

    #[external(v0)]
    fn bad_sub_controlled(ref self:ContractState, user_param:felt252) {
        let a = self.a.read();
        self.b.write(a - user_param);
    }

}