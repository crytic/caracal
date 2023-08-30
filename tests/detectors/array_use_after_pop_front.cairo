#[starknet::contract]
mod ArrayUseAfterPopFront {
    use array::ArrayTrait;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ArrayEvent: ArrayEvent
    }

    #[derive(Drop, starknet::Event)]
    struct ArrayEvent {
        arr: Array<u128>
    }

    #[external(v0)]
    fn bad(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let b = arr.pop_front();
        self.emit(ArrayEvent{ arr});
    }

    #[external(v0)]
    fn bad_one_branch(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        match arr.pop_front() {
            Option::Some(val) => {
                self.emit(ArrayEvent{ arr });
                ()
            },
            Option::None(_) => ()
        };
    }

    #[external(v0)]
    fn bad_loop(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        loop {
            match arr.pop_front() {
                Option::Some(val) => (),
                Option::None(_) => {
                    break ();
                }
            };
        };

        self.emit(ArrayEvent{ arr });
    }

    #[external(v0)]
    fn bad_return(ref self: ContractState) -> Array<u128> {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let b = arr.pop_front();
        return arr;
    }

    #[external(v0)]
    fn good(self: @ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        arr.pop_front();
    }

    #[external(v0)]
    fn good_loop(self: @ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        loop {
            match arr.pop_front() {
                Option::Some(val) => (),
                Option::None(_) => {
                    break ();
                }
            };
        };
    }
}