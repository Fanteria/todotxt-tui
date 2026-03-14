use crate::{config::FileWorkerConfig, file_worker::file_format::FileFormatTrait, todo::ToDo};
use anyhow::{Context, Result};
use std::{
    arch,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use todo_txt::task::Simple as Task;

fn load_from_reader<R: Read>(reader: R, todo: &mut ToDo) -> Result<()> {
    for line in BufReader::new(reader).lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match Task::from_str(line) {
            Ok(task) => todo.add_task(task),
            Err(e) => log::warn!("Task cannot be load due {e}: {line}"),
        }
    }
    Ok(())
}

fn save_to_writer<W: Write>(writer: &mut W, tasks: &[Task]) -> Result<()> {
    let mut writer = BufWriter::new(writer);
    for task in tasks.iter() {
        writer.write_all((task.to_string() + "\n").as_bytes())?;
    }
    Ok(())
}

pub struct TodoTxt {
    path: PathBuf,
    archive: Option<PathBuf>,
}

impl TodoTxt {
    pub fn new(config: &FileWorkerConfig) -> Self {
        Self {
            path: config.todo_path.clone(),
            archive: config.archive_path.clone(),
        }
    }
}

impl FileFormatTrait for TodoTxt {
    fn load_tasks(&self, todo: &mut ToDo) -> Result<()> {
        load_tasks(&self.path, todo)?;
        if let Some(archive) = &self.archive {
            load_tasks(archive, todo)?;
        }
        Ok(())
    }

    fn save_tasks(&self, todo: &mut ToDo) -> Result<()> {
        match &self.archive {
            Some(archive) => {
                save_tasks(&self.path, &todo.pending)?;
                save_tasks(archive, &todo.done)
            }
            None => {
                let mut all = todo.pending.clone();
                all.extend_from_slice(&todo.done);
                save_tasks(&self.path, &all)
            }
        }
    }
}

fn load_tasks(path: &Path, todo: &mut ToDo) -> Result<()> {
    let file = File::open(path).with_context(|| format!("{path:?}"))?;
    load_from_reader(file, todo)
}

fn save_tasks(path: &Path, tasks: &[Task]) -> Result<()> {
    let file = File::create(path).with_context(|| format!("{path:?}"))?;
    save_to_writer(&mut { file }, tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTING_STRING: &str = r#"
        x (A) 2023-05-21 2023-04-30 measure space for 1 +project1 @context1 #hashtag1 due:2023-06-30
                         2023-04-30 measure space for 2 +project2 @context2           due:2023-06-30
                     (C) 2023-04-30 measure space for 3 +project3 @context3           due:2023-06-30
                                    measure space for 4 +project2 @context3 #hashtag1 due:2023-06-30
                                  x measure space for 5 +project3 @context3 #hashtag2 due:2023-06-30
                                    measure space for 6 +project3 @context2 #hashtag2 due:2023-06-30
        "#;

    #[test]
    fn test_load_tasks() -> Result<()> {
        let mut todo = ToDo::default();
        load_from_reader(TESTING_STRING.as_bytes(), &mut todo)?;
        assert_eq!(todo.pending.len(), 4);
        assert_eq!(todo.done.len(), 2);
        assert_eq!(
            todo.pending[0].subject,
            "measure space for 2 +project2 @context2"
        );
        assert_eq!(
            todo.pending[1].subject,
            "measure space for 3 +project3 @context3"
        );
        assert_eq!(todo.pending[1].priority, 2);
        assert_eq!(
            todo.pending[2].subject,
            "measure space for 4 +project2 @context3 #hashtag1"
        );
        assert_eq!(
            todo.pending[3].subject,
            "measure space for 6 +project3 @context2 #hashtag2"
        );
        assert_eq!(
            todo.done[0].subject,
            "measure space for 1 +project1 @context1 #hashtag1"
        );
        assert_eq!(
            todo.done[1].subject,
            "measure space for 5 +project3 @context3 #hashtag2"
        );
        Ok(())
    }

    #[test]
    fn test_save_tasks() -> Result<()> {
        let mut todo = ToDo::default();
        load_from_reader(TESTING_STRING.as_bytes(), &mut todo)?;
        let get_expected = |line: fn(&String) -> bool| {
            TESTING_STRING
                .trim()
                .lines()
                .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(line)
                .collect::<Vec<String>>()
                .join("\n")
                + "\n"
        };
        let pretty_assert = |tasks, expected: &str, msg: &str| -> Result<()> {
            let mut buf: Vec<u8> = Vec::new();
            save_to_writer(&mut buf, tasks)?;
            assert_eq!(
                expected.as_bytes(),
                buf,
                "\n-----{}-----\nGET:\n{}\n----------------\nEXPECTED:\n{}\n",
                msg,
                String::from_utf8(buf.clone()).unwrap(),
                expected
            );
            Ok(())
        };

        pretty_assert(
            &todo.pending,
            &get_expected(|line| !line.starts_with("x ")),
            "Pending check is wrong",
        )?;
        pretty_assert(
            &todo.done,
            &get_expected(|line| line.starts_with("x ")),
            "Done check is wrong",
        )?;

        Ok(())
    }
}
