use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use crate::core::function::Type;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;
use std::collections::HashSet;

#[derive(Default)]
pub struct UnusedEvents {}

impl Detector for UnusedEvents {
    fn name(&self) -> &str {
        "unused-events"
    }

    fn description(&self) -> &str {
        "Detect events defined but not emitted"
    }

    fn confidence(&self) -> Confidence {
        Confidence::Medium
    }

    fn impact(&self) -> Impact {
        Impact::Medium
    }

    fn run(&self, core: &CoreUnit) -> Vec<Result> {
        let mut results = Vec::new();
        let compilation_units = core.get_compilation_units();

        for compilation_unit in compilation_units {
            let mut events: HashSet<String> = compilation_unit
                .functions()
                .filter(|f| *f.ty() == Type::Event)
                .map(|f| f.name())
                .collect();

            for f in compilation_unit.functions_user_defined() {
                for event_stmt in f.events_emitted() {
                    if let SierraStatement::Invocation(invoc) = event_stmt {
                        // Get the concrete libfunc called
                        let libfunc = compilation_unit
                            .registry()
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        if let CoreConcreteLibfunc::FunctionCall(f_called) = libfunc {
                            // We remove the function called from events
                            events.remove(
                                &f_called
                                    .function
                                    .id
                                    .debug_name
                                    .as_ref()
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }
                }
            }

            // We rsplit the event function to get the function name and the first part is the module where the event is defined
            events
                .iter()
                .map(|event_function| event_function.rsplit_once("::").unwrap())
                .for_each(|(event_declaration, event_name)| {
                    results.push(Result {
                        name: self.name().to_string(),
                        impact: self.impact(),
                        confidence: self.confidence(),
                        message: format!(
                            "Event {} defined in {} is never emitted",
                            event_name, event_declaration
                        ),
                    })
                });
        }
        results
    }
}
