#[contract]
mod Felt252Overflow {

    struct Storage {
        a: felt252,
        b: felt252
    }

    #[external]
    fn bad_mul_controlled(param1: felt252, param2: felt252, param3: felt252) -> felt252 {
        let max: felt252 = param1 * param2;
        let my: felt252 = param1 * param1;
        b::write(max * param3);
        return a::read();
    }

    #[external]
    fn bad_add_controlled(user_param: felt252, user_param2: felt252) {
        let a = a::read();
        let controlled = bad_add(user_param, user_param2);
        b::write(a + user_param+ user_param2);
    }

    #[external]
    fn bad_sub_uncontrolled()  {
        let min: felt252 =0;
        a::write(min-1);
    }

    #[view]
    fn bad_add_uncontrolled() -> felt252 {
        let max: felt252 = 0 - 1;
        bad_add(max,2)
    }

    fn bad_add(param1: felt252, param2: felt252) -> felt252{
        test_assert(param1);
        param1 + param2
    }

    #[external]
    fn bad_sub_controlled(user_param: felt252) {
        let a = a::read();
        b::write(a - user_param);
    }
    
    #[view]
    fn test_sub_assert(p: felt252) -> felt252 {
        test_assert(p);
        if p == 10 {
            3
        }
        else {
            p - 5
        }
    }
    
    fn test_assert(p: felt252) {
        assert(4 != 0,'bad');
        assert(p == 3, 'ok');
    }

}