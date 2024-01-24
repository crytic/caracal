#[starknet::interface]
trait IAnotherContract<T> {
    fn foo(ref self: T, a: u128) -> u128;
}

#[starknet::contract]
mod TestContract {
    use super::IAnotherContractDispatcherTrait;
    use super::IAnotherContractLibraryDispatcher;
    use starknet::class_hash::ClassHash;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn bad1(ref self: ContractState, class_hash: ClassHash) -> u128 {
       IAnotherContractLibraryDispatcher { class_hash: class_hash }.foo(2_u128)
    }

    #[external(v0)]
    fn bad2(ref self: ContractState, class_hash: ClassHash) -> u128 {
        let _a = 2_u128; // Need this otherwise the compiler inline this function in the wrapper
        internal_lib_call(class_hash)
    }

    fn internal_lib_call(class_hash: ClassHash) -> u128 {
        IAnotherContractLibraryDispatcher { class_hash: class_hash }.foo(2_u128)
    }

    #[external(v0)]
    fn bad3(ref self: ContractState, class_hash: ClassHash) -> u128 {
        let _a = 2_u128; // Need this otherwise the compiler inline this function in the wrapper
        internal_lib_call_1(class_hash)
    }

    fn internal_lib_call_1(class_hash: ClassHash) -> u128 {
        internal_lib_call(class_hash)
    }


    #[external(v0)]
    fn good(ref self: ContractState) -> u128 {
        IAnotherContractLibraryDispatcher { class_hash: starknet::class_hash_const::<0>() }.foo(2_u128)
    }

}
