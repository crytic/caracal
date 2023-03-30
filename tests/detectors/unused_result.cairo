use option::OptionTrait;

#[contract]
mod UnusedResult {
    struct Storage {
        value: felt252,
    }

    #[external]
    fn unused_result_1(amount: felt252) {
        add_1(amount);
    }

    #[external]
    fn unused_result_2(amount: felt252) {
        add_2(amount);
    }

    #[external]
    fn unused_result_3(amount: felt252) {
        add_3(amount);
    }

    #[external]
    fn unused_result_4(amount: felt252) {
        add_4(amount);
    }

    fn add_1(amount: felt252) -> felt252 {
        value::write(amount);
        23
    }

    fn add_2(amount: felt252) -> (felt252, felt252) {
        (amount, amount)
    }

    fn add_3(amount: felt252) -> felt252 {
        amount
    }

    fn add_4(amount: felt252) -> Option::<felt252> {
        Option::Some(amount)
    }

}
