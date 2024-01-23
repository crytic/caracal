use cairo_lang_sierra::program::{GenStatement, Statement as SierraStatement};
use fxhash::{FxHashMap, FxHashSet};

/// Wrapper around a VarId
/// it's used to univocally identify variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WrapperVariable {
    /// The function where the variable appear
    function: String,
    /// The variable's id
    variable: u64,
}

impl WrapperVariable {
    pub fn new(function: String, variable: u64) -> Self {
        WrapperVariable { function, variable }
    }

    /// Return the function's name where this variable is present
    pub fn function(&self) -> &String {
        &self.function
    }

    /// Return the variable
    pub fn variable(&self) -> u64 {
        self.variable
    }
}

#[derive(Debug, Clone, Default)]
pub struct Taint {
    /// Source WrapperVariable to set of sink WrapperVariable
    /// e.g. instruction reads variables 3, 4 and has 5 as output
    /// we will add (3, (5)) and (4, (5)); variable 5 is tainted by 3 and 4
    map: FxHashMap<WrapperVariable, FxHashSet<WrapperVariable>>,
}

impl Taint {
    pub fn new(instructions: &[SierraStatement], function: String) -> Self {
        let mut map = FxHashMap::default();
        analyze(&mut map, instructions, function);

        Taint { map }
    }

    /// Returns variables tainted in a single step by source
    pub fn single_step_taint(&self, source: &WrapperVariable) -> FxHashSet<WrapperVariable> {
        self.map.get(source).cloned().unwrap_or_default()
    }

    /// Returns variables tainted in zero or more steps by source
    pub fn multi_step_taint(&self, source: &WrapperVariable) -> FxHashSet<WrapperVariable> {
        let mut result = FxHashSet::default();
        let mut update = FxHashSet::from_iter([source.clone()]);
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
        sinks: &FxHashSet<WrapperVariable>,
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
        sinks: &FxHashSet<WrapperVariable>,
    ) -> bool {
        self.multi_step_taint(source)
            .iter()
            .any(|sink| sinks.contains(sink))
    }

    /// Returns the sources that taint the sink
    pub fn taints_any_sources_variable(
        &self,
        sources: &FxHashSet<WrapperVariable>,
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
        sources: &FxHashSet<WrapperVariable>,
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
    map: &mut FxHashMap<WrapperVariable, FxHashSet<WrapperVariable>>,
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
                            variable: source.id,
                        })
                        .or_default();
                    sinks.insert(WrapperVariable {
                        function: function.clone(),
                        variable: sink.id,
                    });
                }
            }
        }
    }
}
