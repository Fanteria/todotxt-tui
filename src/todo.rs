use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
use todo_txt::Task;

#[allow(dead_code)]
struct ToDo {
    pending: Vec<Task>,
    done: Vec<Task>,
}

#[allow(dead_code)]
impl ToDo {
    pub fn load<P>(file_path: P) -> Result<ToDo, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut pending = Vec::new();
        let mut done = Vec::new();

        for line in BufReader::new(File::open(file_path)?).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let task = Task::from_str(&line)?;
            if task.finished {
                done.push(task);
            } else {
                pending.push(task);
            }
        }

        Ok(ToDo { pending, done })
    }
}

#[cfg(test)]
mod tests {
    use super::ToDo;
    use std::error::Error;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::{Result as ioResult, Write};
    use std::path::Path;

    fn test_path(filename: &str) -> String {
        String::from(env!("CARGO_MANIFEST_DIR"))
            + "/resources/test/tmp/"
            + "todo_test/"
            + filename
            + ".conf"
    }

    fn write_to_test_file(filename: &str, content: &str) -> ioResult<()> {
        if Path::new(&test_path(filename)).exists() {
            fs::remove_file(test_path(filename))?;
        }
        let path_string = test_path(filename);
        let path = Path::new(&path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();

        let mut f = OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(path_string)?;
        f.write(content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_load() -> Result<(), Box<dyn Error>> {
        write_to_test_file(
            "test_load",
            r#"
        x (A) 2023-05-21 2023-04-30 measure space for +project1 @context1 #hashtag1 due:2023-06-30
                         2023-04-30 measure space for +project2 @context2           due:2023-06-30
          (C) 2023-04-30 measure space for +project3 @context3           due:2023-06-30
        "#,
        )?;

        let todo = ToDo::load(test_path("test_load"))?;

        assert_eq!(todo.done.len(), 1);
        assert_eq!(todo.pending.len(), 2);

        assert_eq!(todo.done[0].priority, 0);
        assert!(todo.done[0].create_date.is_some());
        assert!(todo.done[0].finish_date.is_some());
        assert_eq!(todo.done[0].finished, true);
        assert_eq!(todo.done[0].threshold_date, None);
        assert!(todo.done[0].due_date.is_some());
        assert_eq!(todo.done[0].contexts.len(), 1);
        assert_eq!(todo.done[0].projects.len(), 1);
        assert_eq!(todo.done[0].hashtags.len(), 1);

        assert!(todo.pending[0].priority.is_lowest());
        assert!(todo.pending[0].create_date.is_some());
        assert!(todo.pending[0].finish_date.is_none());
        assert_eq!(todo.pending[0].finished, false);
        assert_eq!(todo.pending[0].threshold_date, None);
        assert!(todo.pending[0].due_date.is_some());
        assert_eq!(todo.pending[0].contexts.len(), 1);
        assert_eq!(todo.pending[0].projects.len(), 1);
        assert_eq!(todo.pending[0].hashtags.len(), 0);

        assert_eq!(todo.pending[1].priority, 2);
        assert!(todo.pending[1].create_date.is_some());
        assert!(todo.pending[1].finish_date.is_none());
        assert_eq!(todo.pending[1].finished, false);
        assert_eq!(todo.pending[1].threshold_date, None);
        assert!(todo.pending[1].due_date.is_some());
        assert_eq!(todo.pending[1].contexts.len(), 1);
        assert_eq!(todo.pending[1].projects.len(), 1);
        assert_eq!(todo.pending[1].hashtags.len(), 0);

        Ok(())
    }
}
