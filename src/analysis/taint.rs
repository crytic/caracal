use cairo_lang_sierra::{
    ids::VarId,
    program::{GenStatement, Statement as SierraStatement},
};
use std::collections::{HashMap, HashSet};

/// Wrapper around a VarId
/// it's used to univocally identify variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WrapperVariable {
    /// The function where the variable appear
    function: String,
    /// The variable's id
    variable: VarId,
}

impl WrapperVariable {
    pub fn new(function: String, variable: VarId) -> Self {
        WrapperVariable { function, variable }
    }

    pub fn function(&self) -> &String {
        &self.function
    }

    pub fn variable(&self) -> &VarId {
        &self.variable
    }
}

#[derive(Debug, Clone, Default)]
pub struct Taint {
    /// Source WrapperVariable to set of sink WrapperVariable
    /// e.g. instruction reads variables 3, 4 and has 5 as output
    /// we will add (3, (5)) and (4, (5)); variable 5 is tainted by 3 and 4
    map: HashMap<WrapperVariable, HashSet<WrapperVariable>>,
}

impl Taint {
    pub fn new(instructions: &[SierraStatement], function: String) -> Self {
        let mut map = HashMap::new();
        analyze(&mut map, instructions, function);

        Taint { map }
    }

    /// Returns variables tainted in a single step by source
    pub fn single_step_taint(&self, source: &WrapperVariable) -> HashSet<WrapperVariable> {
        self.map.get(source).cloned().unwrap_or_default()
    }

    /// Returns variables tainted in zero or more steps by source
    pub fn multi_step_taint(&self, source: &WrapperVariable) -> HashSet<WrapperVariable> {
        let mut result = HashSet::new();
        let mut update = HashSet::from([source.clone()]);
        while !update.is_subset(&result) {
            result.extend(update.iter().cloned());
            update = update
                .iter()
                .flat_map(|source| self.single_step_taint(source))
                .collect();
        }
        result
    }

    /// Returns the sink variables tainted by the source
    pub fn taints_any_sinks_variable(
        &self,
        source: &WrapperVariable,
        sinks: &HashSet<WrapperVariable>,
    ) -> Vec<WrapperVariable> {
        self.multi_step_taint(source)
            .into_iter()
            .filter(|sink| sinks.contains(sink))
            .collect()
    }

    /// Returns true if the source taints any of the sinks
    pub fn taints_any_sinks(
        &self,
        source: &WrapperVariable,
        sinks: &HashSet<WrapperVariable>,
    ) -> bool {
        self.multi_step_taint(source)
            .iter()
            .any(|sink| sinks.contains(sink))
    }

    /// Returns the sources that taint the sink
    pub fn taints_any_sources_variable(
        &self,
        sources: &HashSet<WrapperVariable>,
        sink: &WrapperVariable,
    ) -> Vec<WrapperVariable> {
        sources
            .clone()
            .into_iter()
            .filter(|source| self.multi_step_taint(source).contains(sink))
            .collect()
    }

    /// Returns true if the sink is tainted by any source
    pub fn taints_any_sources(
        &self,
        sources: &HashSet<WrapperVariable>,
        sink: &WrapperVariable,
    ) -> bool {
        sources
            .iter()
            .any(|source| self.multi_step_taint(source).contains(sink))
    }

    /// Add a taint from source to sink, return true if the sink was new
    pub fn add_taint(&mut self, source: WrapperVariable, sink: WrapperVariable) -> bool {
        let sinks = self.map.entry(source).or_default();
        sinks.insert(sink)
    }
}

/// Analyze each instruction in the current function and populate the taint map
fn analyze(
    map: &mut HashMap<WrapperVariable, HashSet<WrapperVariable>>,
    instructions: &[SierraStatement],
    function: String,
) {
    for instruction in instructions.iter() {
        // We only care about GenStatement::Invocation because GenStatement::Return doesn't add any taint
        if let GenStatement::Invocation(inv) = instruction {
            let mut vars_written = Vec::new();
            let vars_read = &inv.args;
            // Branches have the results which are the variable in output (written)
            for branch in inv.branches.iter() {
                vars_written.extend(branch.results.clone());
            }
            // We add for each variable written all the variables read as taint
            for sink in vars_written.iter() {
                for source in vars_read.iter() {
                    let sinks = map
                        .entry(WrapperVariable {
                            function: function.clone(),
                            variable: source.clone(),
                        })
                        .or_default();
                    sinks.insert(WrapperVariable {
                        function: function.clone(),
                        variable: sink.clone(),
                    });
                }
            }
        }
    }
}
