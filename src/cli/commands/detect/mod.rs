use super::Cmd;
use caracal::{
    core::core_unit::{CoreOpts, CoreUnit},
    detectors::{detector::Impact, detector::Result, get_detectors},
};
use clap::{Args, ValueHint};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct DetectArgs {
    /// Target to analyze
    #[arg(value_hint = ValueHint::FilePath)]
    target: PathBuf,

    /// Corelib path (e.g. mypath/corelib/src)
    #[arg(long)]
    corelib: Option<PathBuf>,

    /// Detectors to run
    #[arg(long, num_args(0..), conflicts_with_all(["exclude", "exclude_informational", "exclude_low", "exclude_medium", "exclude_high"]))]
    detect: Option<Vec<String>>,

    /// Detectors to exclude
    #[arg(long, num_args(0..))]
    exclude: Option<Vec<String>>,

    /// Exclude detectors with informational impact
    #[arg(long)]
    exclude_informational: bool,

    /// Exclude detectors with low impact
    #[arg(long)]
    exclude_low: bool,

    /// Exclude detectors with medium impact
    #[arg(long)]
    exclude_medium: bool,

    /// Exclude detectors with high impact
    #[arg(long)]
    exclude_high: bool,
}

impl From<&DetectArgs> for CoreOpts {
    fn from(args: &DetectArgs) -> Self {
        CoreOpts {
            target: args.target.clone(),
            corelib: args.corelib.clone(),
        }
    }
}

impl Cmd for DetectArgs {
    fn run(&self) -> anyhow::Result<()> {
        let core = CoreUnit::new(self.into())?;
        let mut detectors = get_detectors();

        if let Some(detectors_to_run) = &self.detect {
            detectors.retain(|d| detectors_to_run.contains(&d.name().to_string()));
        } else {
            if let Some(detectors_to_exclude) = &self.exclude {
                detectors.retain(|d| !detectors_to_exclude.contains(&d.name().to_string()));
            }

            if self.exclude_informational {
                detectors.retain(|d| d.impact() != Impact::Informational);
            }

            if self.exclude_low {
                detectors.retain(|d| d.impact() != Impact::Low);
            }

            if self.exclude_medium {
                detectors.retain(|d| d.impact() != Impact::Medium);
            }

            if self.exclude_high {
                detectors.retain(|d| d.impact() != Impact::High);
            }
        }

        let mut results = detectors
            .iter()
            .flat_map(|d| d.run(&core))
            .collect::<Vec<Result>>();
        results.sort();

        results.iter().for_each(|r| println!("{r}"));

        Ok(())
    }
}
