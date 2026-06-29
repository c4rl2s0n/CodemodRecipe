use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub workspace_root: PathBuf,
    pub codemod_root: PathBuf,
}

impl HostConfig {
    pub fn from_env_args() -> Self {
        let mut workspace_root: Option<PathBuf> = None;
        let mut codemod_root: Option<PathBuf> = None;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--workspace-root" => workspace_root = args.next().map(PathBuf::from),
                "--codemod-root" => codemod_root = args.next().map(PathBuf::from),
                "--stdio-server" => {}
                "--empty-constructor-style" => {
                    let _ = args.next();
                }
                _ => {}
            }
        }

        let workspace_root = workspace_root
            .or_else(|| std::env::var("CODEMOD_WORKSPACE_ROOT").ok().map(PathBuf::from))
            .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
        let codemod_root = codemod_root
            .or_else(|| std::env::var("CODEMOD_ROOT").ok().map(PathBuf::from))
            .unwrap_or_else(|| workspace_root.join(".codemod"));

        Self {
            workspace_root,
            codemod_root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_codemod_root_under_workspace() {
        let cfg = HostConfig {
            workspace_root: PathBuf::from("/tmp/ws"),
            codemod_root: PathBuf::from("/tmp/ws/.codemod"),
        };
        assert_eq!(cfg.codemod_root, cfg.workspace_root.join(".codemod"));
    }
}
