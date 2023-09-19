#[starknet::interface]
trait IAnotherContract<T> {
    fn foo(ref self: T, a: Array<u128>) -> u128;
}

#[starknet::contract]
mod ArrayUseAfterPopFront {
    use super::{
        IAnotherContractDispatcherTrait,
        IAnotherContractDispatcher,
        IAnotherContractLibraryDispatcher
    };
    use array::ArrayTrait;
    use starknet::ContractAddress;

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
    fn bad_library_call(ref self: ContractState) -> u128 {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let b = arr.pop_front();
        return IAnotherContractLibraryDispatcher { class_hash: starknet::class_hash_const::<0>() }.foo(arr);
    }

    #[external(v0)]
    fn bad_external_call(ref self: ContractState) -> u128 {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let b = arr.pop_front();
        return IAnotherContractDispatcher { contract_address: starknet::contract_address_const::<0>() }.foo(arr);
    }

    #[external(v0)]
    fn bad_multiple_arrays(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let b = arr.pop_front();
        self.emit(ArrayEvent{ arr });

        let mut arr1 = ArrayTrait::<u128>::new();
        arr1.append(1);

        let b1 = arr1.pop_front();
        self.emit(ArrayEvent{ arr: arr1 });
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