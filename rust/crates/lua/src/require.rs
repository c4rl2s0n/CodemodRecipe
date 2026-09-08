//! Sandboxed Luau `require` constrained to `.codemod/` library modules.

use std::collections::VecDeque;
use std::io::Result as IoResult;
use std::path::{Component, Path, PathBuf};
use std::result::Result as StdResult;

use mlua::{Error, Function, Lua, NavigateError, Require, Result as LuaResult, TextRequirer, Value};

use crate::load::{assert_library_module, is_declaration_file_name, LoadError};

/// Chunk name for a file under `.codemod/` so Luau require-by-string allows `require`.
pub fn require_chunk_name(path: &Path) -> String {
    format!("@{}", path.display())
}

/// Install sandboxed `require` that only loads Library-shaped modules under `codemod_root`.
///
/// Call **before** `lua.sandbox(true)` so the global can be replaced.
pub fn install_codemod_require(lua: &Lua, codemod_root: &Path) -> Result<(), LoadError> {
    let root = codemod_root
        .canonicalize()
        .unwrap_or_else(|_| codemod_root.to_path_buf());
    let require = lua
        .create_require_function(CodemodRequire::new(root))
        .map_err(|e| LoadError::Message(format!("failed to create require: {e}")))?;
    lua.globals()
        .set("require", require)
        .map_err(|e| LoadError::Message(format!("failed to install require: {e}")))?;
    Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = VecDeque::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(..) | Component::RootDir => {
                components.push_back(comp);
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(components.back(), None | Some(Component::ParentDir)) {
                    components.push_back(Component::ParentDir);
                } else if matches!(components.back(), Some(Component::Normal(..))) {
                    components.pop_back();
                }
            }
            Component::Normal(..) => components.push_back(comp),
        }
    }
    components.into_iter().collect()
}

struct CodemodRequire {
    inner: TextRequirer,
    root: PathBuf,
    /// Absolute path of the current navigation context (module or directory).
    abs_path: PathBuf,
}

impl CodemodRequire {
    fn new(root: PathBuf) -> Self {
        Self {
            inner: TextRequirer::new(),
            root,
            abs_path: PathBuf::new(),
        }
    }

    fn strip_chunk_prefix(chunk_name: &str) -> &str {
        let name = chunk_name.strip_prefix('@').unwrap_or(chunk_name);
        if let Some((path, line)) = name.rsplit_once(':') {
            if line.parse::<u32>().is_ok() {
                return path;
            }
        }
        name
    }

    fn path_under_root(&self, path: &Path) -> bool {
        let root = self.root.canonicalize().unwrap_or_else(|_| self.root.clone());
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let normalized = normalize_path(&abs);
        let resolved = normalized.canonicalize().unwrap_or(normalized);
        resolved.starts_with(&root)
    }

    fn ensure_under_root(&self, path: &Path) -> StdResult<(), NavigateError> {
        if self.path_under_root(path) {
            Ok(())
        } else {
            Err(NavigateError::Other(Error::runtime(
                "require path escapes .codemod/",
            )))
        }
    }

    fn strip_lua_extension(path: &Path) -> PathBuf {
        match path.extension().and_then(|e| e.to_str()) {
            Some("luau") | Some("lua") => path.with_extension(""),
            _ => path.to_path_buf(),
        }
    }

    fn sync_from_inner_reset(&mut self, chunk_name: &str) -> StdResult<(), NavigateError> {
        let stripped = Self::strip_chunk_prefix(chunk_name);
        let path = PathBuf::from(stripped);
        let abs = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .map_err(|_| NavigateError::NotFound)?
                .join(&path)
        };
        // Prefer existing file location for root checks; navigate without extension
        // so TextRequirer resolve_module can find `.luau` / `.lua`.
        let abs_for_check = abs.canonicalize().unwrap_or_else(|_| abs.clone());
        self.ensure_under_root(&abs_for_check)?;
        let nav = Self::strip_lua_extension(&abs_for_check);
        let with_at = format!("@{}", nav.display());
        self.inner.reset(&with_at)?;
        self.abs_path = nav;
        Ok(())
    }
}

impl Require for CodemodRequire {
    fn is_require_allowed(&self, chunk_name: &str) -> bool {
        // Luau require-by-string only invokes this for the *requiring* chunk.
        // Accept `@path` and bare paths that resolve under `.codemod/`.
        let stripped = Self::strip_chunk_prefix(chunk_name);
        let path = PathBuf::from(stripped);
        let abs = if path.is_absolute() {
            path
        } else {
            match std::env::current_dir() {
                Ok(cwd) => cwd.join(path),
                Err(_) => return false,
            }
        };
        self.path_under_root(&abs)
    }

    fn reset(&mut self, chunk_name: &str) -> StdResult<(), NavigateError> {
        self.sync_from_inner_reset(chunk_name)
    }

    fn jump_to_alias(&mut self, path: &str) -> StdResult<(), NavigateError> {
        let p = PathBuf::from(path);
        self.ensure_under_root(&p)?;
        self.inner.jump_to_alias(path)?;
        self.abs_path = Self::strip_lua_extension(&p.canonicalize().unwrap_or(p));
        Ok(())
    }

    fn to_parent(&mut self) -> StdResult<(), NavigateError> {
        let mut next = self.abs_path.clone();
        if !next.pop() {
            return Err(NavigateError::NotFound);
        }
        self.ensure_under_root(&next)?;
        self.inner.to_parent()?;
        self.abs_path = next;
        Ok(())
    }

    fn to_child(&mut self, name: &str) -> StdResult<(), NavigateError> {
        let next = self.abs_path.join(name);
        self.ensure_under_root(&next)?;
        self.inner.to_child(name)?;
        self.abs_path = next;
        Ok(())
    }

    fn has_module(&self) -> bool {
        self.inner.has_module()
    }

    fn cache_key(&self) -> String {
        self.inner.cache_key()
    }

    fn has_config(&self) -> bool {
        self.inner.has_config()
    }

    fn config(&self) -> IoResult<Vec<u8>> {
        self.inner.config()
    }

    fn loader(&self, lua: &Lua) -> LuaResult<Function> {
        let key = self.cache_key();
        let path = PathBuf::from(&key);
        if is_declaration_file_name(&path) {
            return Err(Error::runtime(
                "cannot require .d.luau / .d.lua declaration files (IDE-only)",
            ));
        }
        if !self.path_under_root(&path) {
            return Err(Error::runtime("require path escapes .codemod/"));
        }
        let inner_loader = self.inner.loader(lua)?;
        lua.create_function(move |_lua, ()| {
            let value: Value = inner_loader.call(())?;
            match value {
                Value::Table(table) => {
                    assert_library_module(&table).map_err(|e| Error::runtime(e.to_string()))?;
                    Ok(Value::Table(table))
                }
                other => Err(Error::runtime(format!(
                    "required module must return a table, got {}",
                    other.type_name()
                ))),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::{eval_return_table, new_sandbox_with_require};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static N: AtomicUsize = AtomicUsize::new(0);

    fn temp_codemod(name: &str) -> PathBuf {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "codemod_require_{name}_{}_{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("recipes")).unwrap();
        root
    }

    #[test]
    fn require_loads_library_helper() {
        let root = temp_codemod("lib_ok");
        std::fs::write(
            root.join("helpers.luau"),
            r#"return {
  contains = function(s, sub)
    return string.find(s, sub, 1, true) ~= nil
  end,
}
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("recipes/use.luau"),
            r#"
local h = require("../helpers")
return {
  id = "use.helper",
  apply = function(c, args)
    assert(h.contains("hello", "ell"))
  end,
}
"#,
        )
        .unwrap();

        let lua = new_sandbox_with_require(&root).unwrap();
        let recipe_path = root.join("recipes/use.luau");
        let source = std::fs::read_to_string(&recipe_path).unwrap();
        let table = eval_return_table(&lua, &source, &require_chunk_name(&recipe_path)).unwrap();
        let apply: mlua::Function = table.get("apply").unwrap();
        apply.call::<()>((mlua::Value::Nil, mlua::Value::Nil)).unwrap();

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn require_rejects_recipe_asset() {
        let root = temp_codemod("reject_recipe");
        std::fs::write(
            root.join("other.luau"),
            r#"return {
  id = "other.recipe",
  apply = function(c) end,
}
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("recipes/use.luau"),
            r#"
local x = require("../other")
return { id = "use", apply = function() end }
"#,
        )
        .unwrap();

        let lua = new_sandbox_with_require(&root).unwrap();
        let recipe_path = root.join("recipes/use.luau");
        let source = std::fs::read_to_string(&recipe_path).unwrap();
        let err = eval_return_table(&lua, &source, &require_chunk_name(&recipe_path))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("cannot require") || err.contains("Recipe"),
            "unexpected err: {err}"
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn require_rejects_path_escape() {
        let root = temp_codemod("escape");
        std::fs::write(
            root.join("recipes/use.luau"),
            r#"
local x = require("../../../etc/passwd")
return { id = "use", apply = function() end }
"#,
        )
        .unwrap();

        let lua = new_sandbox_with_require(&root).unwrap();
        let recipe_path = root.join("recipes/use.luau");
        let source = std::fs::read_to_string(&recipe_path).unwrap();
        let err = eval_return_table(&lua, &source, &require_chunk_name(&recipe_path))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("escapes") || err.contains("NotFound") || err.contains("could not"),
            "unexpected err: {err}"
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
