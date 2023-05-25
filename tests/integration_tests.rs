use starknet_static_analysis::core::core_unit::{CoreOpts, CoreUnit};
use starknet_static_analysis::detectors::{detector::Result, get_detectors};
use std::env;
use std::path::PathBuf;

#[test]
fn test_detectors() {
    insta::glob!("detectors/", "*.cairo", |path| {
        let opts = CoreOpts {
            target: path.to_path_buf(),
            corelib: Some(PathBuf::from(
                env::var("CARGO_MANIFEST_DIR").unwrap() + "/corelib/src",
            )),
        };
        let core = CoreUnit::new(opts).unwrap();
        insta::assert_debug_snapshot!(get_detectors()
            .iter()
            .flat_map(|d| d.run(&core))
            .collect::<Vec<Result>>());
    });
}
