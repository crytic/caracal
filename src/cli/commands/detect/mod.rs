use super::Cmd;
use caracal::{
    core::core_unit::{CoreOpts, CoreUnit},
    detectors::{detector::Impact, detector::Result, get_detectors},
};
use clap::{Args, ValueHint};
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Args, Debug)]
pub struct DetectArgs {
    /// Target to analyze
    #[arg(value_hint = ValueHint::FilePath)]
    target: PathBuf,

    /// Corelib path (e.g. mypath/corelib/src)
    #[arg(long)]
    corelib: Option<PathBuf>,

    /// Path to the contracts to compile when using a cairo project with multiple contracts
    #[arg(long, num_args(0..))]
    contract_path: Option<Vec<String>>,

    /// Functions name that are safe when called (e.g. they don't cause a reentrancy)
    #[arg(long, num_args(0..))]
    safe_external_calls: Option<Vec<String>>,

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
            contract_path: args.contract_path.clone(),
            safe_external_calls: args.safe_external_calls.clone(),
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

        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        for r in results.iter() {
            match r.impact {
                Impact::High => {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_intense(true))?;
                    writeln!(&mut stdout, "{}", r)?;
                }
                Impact::Medium => {
                    stdout.set_color(
                        ColorSpec::new()
                            .set_fg(Some(Color::Yellow))
                            .set_intense(true),
                    )?;
                    writeln!(&mut stdout, "{}", r)?;
                }
                Impact::Low => {
                    stdout.set_color(
                        ColorSpec::new()
                            .set_fg(Some(Color::Green))
                            .set_intense(true),
                    )?;
                    writeln!(&mut stdout, "{}", r)?;
                }
                Impact::Informational => {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_intense(true))?;
                    writeln!(&mut stdout, "{}", r)?;
                }
            }
        }

        stdout.reset()?;

        Ok(())
    }
}
