use self::detector::Detector;

pub mod controlled_library_call;
pub mod dead_code;
pub mod detector;
pub mod reentrancy;
pub mod reentrancy_benign;
pub mod reentrancy_events;
pub mod unchecked_l1_handler_from;
pub mod unenforced_view;
pub mod unused_arguments;
pub mod unused_events;
pub mod unused_return;

pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::<controlled_library_call::ControlledLibraryCall>::default(),
        Box::<unused_events::UnusedEvents>::default(),
        Box::<dead_code::DeadCode>::default(),
        Box::<unused_arguments::UnusedArguments>::default(),
        Box::<unused_return::UnusedReturn>::default(),
        Box::<unenforced_view::UnenforcedView>::default(),
        Box::<reentrancy_benign::ReentrancyBenign>::default(),
        Box::<reentrancy::Reentrancy>::default(),
        Box::<reentrancy_events::ReentrancyEvents>::default(),
        Box::<unchecked_l1_handler_from::UncheckedL1HandlerFrom>::default(),
    ]
}
