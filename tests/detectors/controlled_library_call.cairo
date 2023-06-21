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
    fn bad2(self: @ContractState, class_hash: ClassHash) -> u128 {
        internal_lib_call(class_hash)
    }

    fn internal_lib_call(class_hash: ClassHash) -> u128 {
        IAnotherContractLibraryDispatcher { class_hash: class_hash }.foo(2_u128)
    }

    #[external(v0)]
    fn good(self: @ContractState) -> u128 {
        IAnotherContractLibraryDispatcher { class_hash: starknet::class_hash_const::<0>() }.foo(2_u128)
    }

}
