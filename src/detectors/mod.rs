use self::detector::{Confidence, Detector, Impact};

pub mod detector;
pub mod unused_return;

pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![Box::new(unused_return::UnusedReturn::new(
        "unused-return".to_string(),
        Impact::Medium,
        Confidence::Medium,
    ))]
}
