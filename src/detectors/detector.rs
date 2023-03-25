use crate::core::core_unit::CoreUnit;

use std::fmt;

pub trait Detector {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn impact(&self) -> Impact;
    fn confidence(&self) -> Confidence;
    fn run(&self, core: &CoreUnit) -> Vec<Result>;
}

impl fmt::Display for dyn Detector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} | Impact: {} | Confidence: {} | {}",
            self.name(),
            self.impact(),
            self.confidence(),
            self.description()
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Impact {
    High,
    Medium,
    Low,
    Informational,
}

impl fmt::Display for Impact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Impact::High => write!(f, "High"),
            Impact::Medium => write!(f, "Medium"),
            Impact::Low => write!(f, "Low"),
            Impact::Informational => write!(f, "Informational"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::High => write!(f, "High"),
            Confidence::Medium => write!(f, "Medium"),
            Confidence::Low => write!(f, "Low"),
        }
    }
}

#[derive(Debug)]
pub struct Result {
    pub name: String,
    pub impact: Impact,
    pub confidence: Confidence,
    pub message: String,
}

impl fmt::Display for Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} Impact: {} Confidence: {}\n {}",
            self.name, self.impact, self.confidence, self.message
        )
    }
}
