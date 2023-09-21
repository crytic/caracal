#[abi]
trait IAnotherContract {
    fn foo(a: Array<felt252>) -> felt252;
}

#[contract]
mod ArrayUseAfterPopFront {
    use super::{
        IAnotherContractDispatcherTrait,
        IAnotherContractDispatcher,
        IAnotherContractLibraryDispatcher
    };
    use array::ArrayTrait;
    use starknet::ContractAddress;

    #[event]
    fn ArrayEvent(arr: Array<felt252>) {}

    #[external]
    fn bad() {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        let b = arr.pop_front();
        ArrayEvent(arr);
    }

    #[external]
    fn bad_one_branch() {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        match arr.pop_front() {
            Option::Some(val) => {
                ArrayEvent(arr);
                ()
            },
            Option::None(_) => ()
        };
    }

    #[external]
    fn bad_loop() {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        loop {
            match arr.pop_front() {
                Option::Some(val) => (),
                Option::None(_) => {
                    break ();
                }
            };
        };

        ArrayEvent(arr);
    }

    #[external]
    fn bad_return() -> Array<felt252> {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        let b = arr.pop_front();
        return arr;
    }

    #[external]
    fn bad_library_call() -> felt252 {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        let b = arr.pop_front();
        return IAnotherContractLibraryDispatcher { class_hash: starknet::class_hash_const::<0>() }.foo(arr);
    }

    #[external]
    fn bad_external_call() -> felt252 {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        let b = arr.pop_front();
        return IAnotherContractDispatcher { contract_address: starknet::contract_address_const::<0>() }.foo(arr);
    }

    #[external]
    fn good() {
        let mut arr = ArrayTrait::<felt252>::new();
        arr.append(1);

        arr.pop_front();
    }

    #[external]
    fn good_loop() {
        let mut arr = ArrayTrait::<felt252>::new();
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