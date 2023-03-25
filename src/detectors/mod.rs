use self::detector::Detector;

pub mod detector;

pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![]
}
