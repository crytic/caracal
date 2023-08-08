#[starknet::contract]
mod UnusedEvents {

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        MyUnusedEvent: MyUnusedEvent,
        MyUsedEvent: MyUsedEvent
    }

    #[derive(Drop, starknet::Event)]
    struct MyUnusedEvent {
        value: u256, 
    }

    #[derive(Drop, starknet::Event)]
    struct MyUsedEvent {
        value: u256, 
    }

    #[external(v0)]
    fn use_event1(ref self: ContractState, amount: u256) {
        self.emit(MyUsedEvent { value: amount });
    }

}
