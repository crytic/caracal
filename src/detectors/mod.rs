use self::detector::Detector;

pub mod array_use_after_pop_front;
pub mod controlled_library_call;
pub mod dead_code;
pub mod detector;
pub mod read_only_reentrancy;
pub mod reentrancy;
pub mod reentrancy_benign;
pub mod reentrancy_events;
pub mod unchecked_l1_handler_from;
pub mod unused_arguments;
pub mod unused_events;
pub mod unused_return;

pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::<array_use_after_pop_front::ArrayUseAfterPopFront>::default(),
        Box::<controlled_library_call::ControlledLibraryCall>::default(),
        Box::<unused_events::UnusedEvents>::default(),
        Box::<dead_code::DeadCode>::default(),
        Box::<unused_arguments::UnusedArguments>::default(),
        Box::<unused_return::UnusedReturn>::default(),
        Box::<reentrancy_benign::ReentrancyBenign>::default(),
        Box::<reentrancy::Reentrancy>::default(),
        Box::<reentrancy_events::ReentrancyEvents>::default(),
        Box::<read_only_reentrancy::ReadOnlyReentrancy>::default(),
        Box::<unchecked_l1_handler_from::UncheckedL1HandlerFrom>::default(),
    ]
}
