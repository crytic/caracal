use super::detector::{Confidence, Detector, Impact, Result};
use crate::core::core_unit::CoreUnit;
use crate::utils::filter_builtins_from_signature;
use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::program::Statement as SierraStatement;
use std::collections::HashSet;

// Note: It's possible to have FPs when the long syntax to emit events is used
// e.g. self.emit(Event::MyUsedEvent(MyUsedEvent { value: amount }));

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
            let mut events: HashSet<String> = compilation_unit.all_events_name().collect();

            for f in compilation_unit.functions_user_defined() {
                for event_stmt in f.events_emitted() {
                    if let SierraStatement::Invocation(invoc) = event_stmt {
                        // Get the concrete libfunc called
                        let libfunc = compilation_unit
                            .registry()
                            .get_libfunc(&invoc.libfunc_id)
                            .expect("Library function not found in the registry");

                        if let CoreConcreteLibfunc::FunctionCall(f_called) = libfunc {
                            // The first non builtin argument is the ContractState, the event is the second
                            let event_name = filter_builtins_from_signature(
                                &f_called.signature.param_signatures,
                            )[1]
                            .ty
                            .debug_name
                            .as_ref()
                            .unwrap()
                            .as_str();
                            // We remove the event emitted from events
                            events.remove(event_name);
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
