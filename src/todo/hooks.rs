use crate::{config::HookPaths, Result, ToDoError};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub enum HookTypes {
    PreNew,
    PostNew,
    PreRemove,
    PostRemove,
    PreMove,
    PostMove,
    PreUpdate,
    PostUpdate,
}

impl Display for HookTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::PreNew => write!(f, "pre new task"),
            Self::PostNew => write!(f, "post new task"),
            Self::PreRemove => write!(f, "pre remove task"),
            Self::PostRemove => write!(f, "post remove task"),
            Self::PreMove => write!(f, "pre move task"),
            Self::PostMove => write!(f, "post move task"),
            Self::PreUpdate => write!(f, "pre update task"),
            Self::PostUpdate => write!(f, "post update task"),
        }
    }
}

#[derive(Default)]
pub struct Hooks {
    paths: HookPaths,
}

impl Hooks {
    pub fn new(paths: HookPaths) -> Self {
        log::debug!("Hooks: {paths:#?}");
        Self { paths }
    }

    fn run_command(path: &Path, task: &str) -> Result<String> {
        let mut cmd = Command::new("bash");
        let path = fs::canonicalize(path)?;
        cmd.arg("--").arg(&path).arg(task);
        if let Some(parent) = path.parent() {
            cmd.current_dir(parent);
        }
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(ToDoError::HookCommandFailed(
                path.to_path_buf(),
                String::from_utf8(output.stderr).map_err(ToDoError::HookFailedToParseError)?,
            ));
        }
        String::from_utf8(output.stdout).map_err(ToDoError::HookFailedToParseStdout)
    }

    fn run_command_with_name(path: &Path, task: &str, name: &str) -> Option<String> {
        log::info!("{name} hook: {path:?} {task}");
        match Self::run_command(path, task) {
            Ok(stdout) => {
                log::debug!("Hook {name} return {stdout}");
                Some(stdout)
            }
            Err(e) => {
                log::error!("Hook post move task failed: {e}");
                None
            }
        }
    }

    fn get_path(&self, hook_type: &HookTypes) -> Option<&PathBuf> {
        match hook_type {
            HookTypes::PreNew => self.paths.pre_new_task.as_ref(),
            HookTypes::PostNew => self.paths.post_new_task.as_ref(),
            HookTypes::PreRemove => self.paths.pre_remove_task.as_ref(),
            HookTypes::PostRemove => self.paths.post_remove_task.as_ref(),
            HookTypes::PreMove => self.paths.pre_move_task.as_ref(),
            HookTypes::PostMove => self.paths.post_move_task.as_ref(),
            HookTypes::PreUpdate => self.paths.pre_update_task.as_ref(),
            HookTypes::PostUpdate => self.paths.post_update_task.as_ref(),
        }
    }

    pub fn run(&self, hook_type: HookTypes, task: impl AsRef<str>) -> Option<String> {
        Self::run_command_with_name(
            self.get_path(&hook_type)?,
            task.as_ref(),
            &format!("pre {hook_type}"),
        )
    }

    pub fn run_lazy(&self, hook_type: HookTypes, task: impl Fn() -> String) -> Option<String> {
        Self::run_command_with_name(
            self.get_path(&hook_type)?,
            task().as_ref(),
            &format!("pre {hook_type}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::var;
    use test_log::test;
    // TODO

    #[test]
    fn run_empty_hooks() {
        let hooks = Hooks::new(HookPaths::default());
        assert_eq!(hooks.run(HookTypes::PreNew, ""), None);
        assert_eq!(hooks.run(HookTypes::PostNew, ""), None);
        assert_eq!(hooks.run(HookTypes::PreRemove, ""), None);
        assert_eq!(hooks.run(HookTypes::PostRemove, ""), None);
        assert_eq!(hooks.run(HookTypes::PreMove, ""), None);
        assert_eq!(hooks.run(HookTypes::PostMove, ""), None);
        assert_eq!(hooks.run(HookTypes::PreUpdate, ""), None);
        assert_eq!(hooks.run(HookTypes::PostUpdate, ""), None);
    }

    #[test]
    fn run_hooks() {
        let path = PathBuf::from(var("TODO_TUI_TEST_DIR").unwrap()).join("hook.sh");
        let hooks = Hooks::new(HookPaths {
            pre_new_task: Some(path.clone()),
            post_new_task: Some(path.clone()),
            pre_remove_task: Some(path.clone()),
            post_remove_task: Some(path.clone()),
            pre_move_task: Some(path.clone()),
            post_move_task: Some(path.clone()),
            pre_update_task: Some(path.clone()),
            post_update_task: Some(path.clone()),
        });

        assert_eq!(
            hooks.run(HookTypes::PreNew, "pre new"),
            Some(String::from("hook: pre new"))
        );
        assert_eq!(
            hooks.run(HookTypes::PostNew, "post new"),
            Some(String::from("hook: post new"))
        );
        assert_eq!(
            hooks.run(HookTypes::PreRemove, "pre remove"),
            Some(String::from("hook: pre remove"))
        );
        assert_eq!(
            hooks.run(HookTypes::PostRemove, "post remove"),
            Some(String::from("hook: post remove"))
        );
        assert_eq!(
            hooks.run(HookTypes::PreMove, "pre move"),
            Some(String::from("hook: pre move"))
        );
        assert_eq!(
            hooks.run(HookTypes::PostMove, "post move"),
            Some(String::from("hook: post move"))
        );
        assert_eq!(
            hooks.run(HookTypes::PreUpdate, "pre update"),
            Some(String::from("hook: pre update"))
        );
        assert_eq!(
            hooks.run(HookTypes::PostUpdate, "post update"),
            Some(String::from("hook: post update"))
        );

        assert_eq!(
            hooks.run_lazy(HookTypes::PreNew, || String::from("pre new")),
            Some(String::from("hook: pre new"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PostNew, || String::from("post new")),
            Some(String::from("hook: post new"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PreRemove, || String::from("pre remove")),
            Some(String::from("hook: pre remove"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PostRemove, || String::from("post remove")),
            Some(String::from("hook: post remove"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PreMove, || String::from("pre move")),
            Some(String::from("hook: pre move"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PostMove, || String::from("post move")),
            Some(String::from("hook: post move"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PreUpdate, || String::from("pre update")),
            Some(String::from("hook: pre update"))
        );
        assert_eq!(
            hooks.run_lazy(HookTypes::PostUpdate, || String::from("post update")),
            Some(String::from("hook: post update"))
        );
    }

    #[test]
    fn invalid_path() {
        let path = PathBuf::from("/this/path/is/not/valid");
        assert!(Hooks::run_command(&path, "").is_err());
        assert_eq!(
            Hooks::run_command_with_name(&path, "task", "cmd name"),
            None
        );

        let path = PathBuf::from(var("TODO_TUI_TEST_DIR").unwrap()).join("hook_failed.sh");

        assert!(Hooks::run_command(&path, "").is_err());
        assert_eq!(
            Hooks::run_command_with_name(&path, "task", "cmd name"),
            None
        );
    }
}
