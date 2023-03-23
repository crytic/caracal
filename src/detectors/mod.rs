use self::detector::{Confidence, Detector, Impact};

pub mod detector;

pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![]
}
