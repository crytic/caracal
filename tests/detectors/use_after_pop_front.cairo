#[starknet::interface]
trait IAnotherContract<T> {
    fn foo(ref self: T, a: Array<u128>) -> u128;
    fn bar(ref self: T, a: Span<u128>) -> u128;
}

#[starknet::contract]
mod UseAfterPopFront {
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
        ArrayEvent: ArrayEvent,
        SpanEvent: SpanEvent
    }

    #[derive(Drop, starknet::Event)]
    struct ArrayEvent {
        arr: Array<u128>
    }

    #[derive(Drop, starknet::Event)]
    struct SpanEvent {
        span: Span<u128>
    }

    #[external(v0)]
    fn bad(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let _b = arr.pop_front();
        self.emit(ArrayEvent{ arr });
    }

    #[external(v0)]
    fn bad_one_branch(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        match arr.pop_front() {
            Option::Some(_val) => {
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
                Option::Some(_val) => (),
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

        let _b = arr.pop_front();
        return arr;
    }

    #[external(v0)]
    fn bad_library_call(ref self: ContractState) -> u128 {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let _b = arr.pop_front();
        return IAnotherContractLibraryDispatcher { class_hash: starknet::class_hash_const::<0>() }.foo(arr);
    }

    #[external(v0)]
    fn bad_external_call(ref self: ContractState) -> u128 {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let _b = arr.pop_front();
        return IAnotherContractDispatcher { contract_address: starknet::contract_address_const::<0>() }.foo(arr);
    }

    #[external(v0)]
    fn bad_multiple_arrays(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let _b = arr.pop_front();
        self.emit(ArrayEvent{ arr });

        let mut arr1 = ArrayTrait::<u128>::new();
        arr1.append(1);

        let _b1 = arr1.pop_front();
        self.emit(ArrayEvent{ arr: arr1 });
    }

    #[external(v0)]
    fn good(self: @ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let _a = arr.pop_front();
    }

    #[external(v0)]
    fn good_loop(self: @ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        loop {
            match arr.pop_front() {
                Option::Some(_val) => (),
                Option::None(_) => {
                    break ();
                }
            };
        };
    }

    // Span test functions
    #[external(v0)]
    fn bad_span(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        let _b = span.pop_front();
        self.emit(SpanEvent{ span });
    }

    #[external(v0)]
    fn bad_one_branch_span(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        match span.pop_front() {
            Option::Some(_val) => {
                self.emit(SpanEvent{ span });
                ()
            },
            Option::None(_) => ()
        };
    }

    #[external(v0)]
    fn bad_loop_span(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        loop {
            match span.pop_front() {
                Option::Some(_val) => (),
                Option::None(_) => {
                    break ();
                }
            };
        };

        self.emit(SpanEvent{ span });
    }

    #[external(v0)]
    fn bad_return_span(ref self: ContractState) -> Span<u128> {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        let _b = span.pop_front();
        return span;
    }

    #[external(v0)]
    fn bad_library_call_span(ref self: ContractState) -> u128 {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        let _b = span.pop_front();
        return IAnotherContractLibraryDispatcher { class_hash: starknet::class_hash_const::<0>() }.bar(span);
    }

    #[external(v0)]
    fn bad_external_call_span(ref self: ContractState) -> u128 {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        let _b = span.pop_front();
        return IAnotherContractDispatcher { contract_address: starknet::contract_address_const::<0>() }.bar(span);
    }

    #[external(v0)]
    fn bad_multiple_arrays_span(ref self: ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        let _b = span.pop_front();
        self.emit(SpanEvent{ span });

        let mut arr1 = ArrayTrait::<u128>::new();
        arr1.append(1);

        let mut span1 = arr.span();
        let _b1 = span1.pop_front();
        self.emit(SpanEvent{ span: span1 });
    }

    #[external(v0)]
    fn good_span(self: @ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        let _a = span.pop_front();
    }

    #[external(v0)]
    fn good_loop_span(self: @ContractState) {
        let mut arr = ArrayTrait::<u128>::new();
        arr.append(1);

        let mut span = arr.span();
        loop {
            match span.pop_front() {
                Option::Some(_val) => (),
                Option::None(_) => {
                    break ();
                }
            };
        };
    }
}