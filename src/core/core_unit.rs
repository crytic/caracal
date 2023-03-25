use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::{setup_project, ProjectConfig, ProjectConfigContent};
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_filesystem::ids::Directory;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::replace_ids::replace_sierra_ids_in_program;
use cairo_lang_starknet::abi::AbiBuilder;
use cairo_lang_starknet::contract::{find_contracts, get_abi};
use cairo_lang_starknet::db::StarknetRootDatabaseBuilderEx;

use crate::core::compilation_unit::CompilationUnit;

pub struct CoreOpts {
    pub file: PathBuf,
    pub corelib: Option<PathBuf>,
}

pub struct CoreUnit {
    compilation_unit: CompilationUnit,
}

impl CoreUnit {
    pub fn new(opts: CoreOpts) -> Result<Self> {
        // corelib cli option has priority over the environment variable
        let corelib = match opts.corelib {
            Some(ref p) => p.clone(),
            None => {
                match env::var("CORELIB_PATH") {
                    Ok(p) => p.into(),
                    Err(e) => bail!("{e}. The Corelib path must be specified either with the CORELIB_PATH environment variable or the --corelib cli option"),
                }
            }
        };

        // Needed to pass the correct corelib path
        let project_config = ProjectConfig {
            corelib: Some(Directory(corelib)),
            base_path: "".into(),
            content: ProjectConfigContent {
                crate_roots: HashMap::new(),
            },
        };

        let mut db = RootDatabase::builder()
            .with_project_config(project_config)
            .with_starknet()
            .build()?;

        let mut compiler_config = CompilerConfig::default();

        let main_crate_ids = setup_project(&mut db, &opts.file)?;
        compiler_config.diagnostics_reporter.ensure(&mut db)?;

        let contracts = find_contracts(&db, &main_crate_ids);
        let contract = match &contracts[..] {
            [contract] => contract,
            [] => bail!("Contract not found."),
            _ => {
                bail!("Compilation unit must include only one contract.",)
            }
        };

        let abi =
            AbiBuilder::from_trait(&db, get_abi(&db, contract)?).with_context(|| "ABI error")?;
        let sierra = db
            .get_sierra_program(main_crate_ids)
            .ok()
            .context("Compilation failed without any diagnostics.")?;
        let sierra = replace_sierra_ids_in_program(&db, &sierra);
        let registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(&sierra)?;
        let mut compilation_unit = CompilationUnit::new(sierra, abi, registry);
        compilation_unit.analyze();

        Ok(CoreUnit { compilation_unit })
    }

    pub fn get_compilation_unit(&self) -> &CompilationUnit {
        &self.compilation_unit
    }
}

#[test]
fn test_detectors() {
    use crate::detectors::{detector::Result, get_detectors};

    insta::glob!("../../tests/detectors", "*.cairo", |path| {
        let opts = CoreOpts {
            file: path.to_path_buf(),
            corelib: Some(PathBuf::from(
                env::var("CARGO_MANIFEST_DIR").unwrap() + "/src/corelib/src",
            )),
        };
        let core = CoreUnit::new(opts).unwrap();
        insta::assert_debug_snapshot!(get_detectors()
            .iter()
            .flat_map(|d| d.run(&core))
            .collect::<Vec<Result>>());
    });
}
