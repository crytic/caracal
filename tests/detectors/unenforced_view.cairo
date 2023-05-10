#[contract]

mod UnenforcedView {
    struct Storage {
        value: felt252,
    }

    #[view]
    fn writes_to_storage(val: felt252) {
        value::write(val);
    }

    #[view]
    fn does_not_write_to_storage() -> felt252 {
        value::read()
    }
}