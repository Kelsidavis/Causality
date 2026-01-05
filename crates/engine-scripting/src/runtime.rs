// Rhai scripting runtime

use anyhow::Result;
use rhai::{Dynamic, Engine, EvalAltResult, Scope, AST};
use std::collections::HashMap;
use std::path::Path;

use engine_scene::entity::EntityId;

/// Script runtime - manages Rhai engine and script execution
pub struct ScriptRuntime {
    engine: Engine,
    scripts: HashMap<EntityId, CompiledScript>,
}

/// Compiled script with AST and scope
pub struct CompiledScript {
    pub ast: AST,
    pub scope: Scope<'static>,
    pub source: String,
}

impl ScriptRuntime {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Configure engine
        engine.set_max_expr_depths(128, 128);
        engine.set_max_operations(100_000);

        Self {
            engine,
            scripts: HashMap::new(),
        }
    }

    /// Load and compile a script from source code
    pub fn load_script(&mut self, entity_id: EntityId, source: String) -> Result<()> {
        let ast = self.engine.compile(&source).map_err(|e| {
            anyhow::anyhow!("Script compilation error for entity {:?}: {}", entity_id, e)
        })?;

        let scope = Scope::new();

        self.scripts.insert(
            entity_id,
            CompiledScript {
                ast,
                scope,
                source,
            },
        );

        Ok(())
    }

    /// Load a script from a file
    pub fn load_script_file<P: AsRef<Path>>(&mut self, entity_id: EntityId, path: P) -> Result<()> {
        let source = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read script file {:?}: {}", path.as_ref(), e))?;

        self.load_script(entity_id, source)
    }

    /// Call a script function (e.g., "update")
    pub fn call_function(
        &mut self,
        entity_id: EntityId,
        function_name: &str,
        args: impl rhai::FuncArgs,
    ) -> Result<Dynamic> {
        let script = self
            .scripts
            .get_mut(&entity_id)
            .ok_or_else(|| anyhow::anyhow!("No script found for entity {:?}", entity_id))?;

        let result = self
            .engine
            .call_fn(&mut script.scope, &script.ast, function_name, args)
            .map_err(|e| anyhow::anyhow!("Script runtime error in {}: {}", function_name, e))?;

        Ok(result)
    }

    /// Evaluate an expression in a script's scope
    pub fn eval_with_scope(&mut self, entity_id: EntityId, expr: &str) -> Result<Dynamic> {
        let script = self
            .scripts
            .get_mut(&entity_id)
            .ok_or_else(|| anyhow::anyhow!("No script found for entity {:?}", entity_id))?;

        let result = self
            .engine
            .eval_with_scope(&mut script.scope, expr)
            .map_err(|e| anyhow::anyhow!("Script eval error: {}", e))?;

        Ok(result)
    }

    /// Remove a script
    pub fn remove_script(&mut self, entity_id: EntityId) -> bool {
        self.scripts.remove(&entity_id).is_some()
    }

    /// Check if an entity has a script
    pub fn has_script(&self, entity_id: EntityId) -> bool {
        self.scripts.contains_key(&entity_id)
    }

    /// Get the engine reference (for registering custom types/functions)
    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    /// Hot reload a script (recompile with same scope)
    pub fn reload_script(&mut self, entity_id: EntityId, source: String) -> Result<()> {
        let ast = self.engine.compile(&source).map_err(|e| {
            anyhow::anyhow!("Script recompilation error for entity {:?}: {}", entity_id, e)
        })?;

        if let Some(script) = self.scripts.get_mut(&entity_id) {
            script.ast = ast;
            script.source = source;
        } else {
            // No existing script, create new one
            self.load_script(entity_id, source)?;
        }

        Ok(())
    }

    /// Get script count
    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }

    /// Clear all scripts
    pub fn clear(&mut self) {
        self.scripts.clear();
    }
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}
