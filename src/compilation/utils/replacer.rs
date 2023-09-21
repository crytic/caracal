use cairo_lang_sierra::debug_info::DebugInfo;
use cairo_lang_sierra_generator::replace_ids::SierraIdReplacer;

pub struct SierraProgramDebugReplacer {
    pub debug_info: DebugInfo,
}

impl SierraIdReplacer for SierraProgramDebugReplacer {
    fn replace_libfunc_id(
        &self,
        id: &cairo_lang_sierra::ids::ConcreteLibfuncId,
    ) -> cairo_lang_sierra::ids::ConcreteLibfuncId {
        let func_name = self
            .debug_info
            .libfunc_names
            .get(id)
            .expect("No libfunc in debug info");
        cairo_lang_sierra::ids::ConcreteLibfuncId {
            id: id.id,
            debug_name: Some(func_name.clone()),
        }
    }

    fn replace_type_id(
        &self,
        id: &cairo_lang_sierra::ids::ConcreteTypeId,
    ) -> cairo_lang_sierra::ids::ConcreteTypeId {
        let type_name = self
            .debug_info
            .type_names
            .get(id)
            .expect("No typeid in debug info");
        cairo_lang_sierra::ids::ConcreteTypeId {
            id: id.id,
            debug_name: Some(type_name.clone()),
        }
    }

    fn replace_function_id(
        &self,
        sierra_id: &cairo_lang_sierra::ids::FunctionId,
    ) -> cairo_lang_sierra::ids::FunctionId {
        let function_name = self
            .debug_info
            .user_func_names
            .get(sierra_id)
            .expect("No funcid in debug info");
        cairo_lang_sierra::ids::FunctionId {
            id: sierra_id.id,
            debug_name: Some(function_name.clone()),
        }
    }
}
