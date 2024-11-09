use crate::config::HookPaths;
use crate::{Result, ToDoError};
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    // TODO
}
