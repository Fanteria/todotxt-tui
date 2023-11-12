use super::ToDo;
use super::ToDoCategory;

fn same_start_index(fst: &str, sec: &str) -> usize {
    for (i, (fst_char, sec_char)) in fst.chars().zip(sec.chars()).enumerate() {
        if fst_char != sec_char {
            return i;
        }
    }
    std::cmp::min(fst.len(), sec.len())
}

/// Handles autocompletion based on user input.
pub fn autocomplete(todo: &ToDo, input: &str) -> Option<String> {
    let last_space_index = input.rfind(' ').map(|i| i + 1).unwrap_or(0);
    let base = input.get(last_space_index..)?;
    let category = base.get(0..1)?;
    let pattern = base.get(1..)?;

    let list = match category {
        "+" => todo.get_categories(ToDoCategory::Projects),
        "@" => todo.get_categories(ToDoCategory::Contexts),
        "#" => todo.get_categories(ToDoCategory::Hashtags),
        _ => return None,
    };

    if list.is_empty() {
        return None;
    }

    let list = list.start_with(pattern);

    if list.is_empty() {
        return None;
    }

    let mut new_act = list[0].as_str();
    if list.len() != 1 {
        list.iter()
            .skip(1)
            .for_each(|item| new_act = &new_act[..same_start_index(new_act, item)]);
        Some(input.to_string() + &new_act[pattern.len()..])
    } else {
        Some(input.to_string() + &new_act[pattern.len()..] + " ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autocomplete_basic() {
        let mut todo = ToDo::new(false);
        todo.new_task("t +project1 +project2").unwrap();
        todo.new_task("t +project1 +project3").unwrap();
        todo.new_task("t +project1 @context1").unwrap();
        todo.new_task("t +project1 #hashtag1").unwrap();

        assert_eq!(autocomplete(&todo, "task"), None);
        assert_eq!(autocomplete(&todo, "task +proj"), Some(String::from("task +project")));
        assert_eq!(autocomplete(&todo, "task +project1"), Some(String::from("task +project1 ")));

    }

    #[test]
    fn autocomplete_empty() {
        let mut todo = ToDo::new(false);
        assert_eq!(autocomplete(&todo, "task +proj"), None);

        todo.new_task("t +project1 +project2").unwrap();
        assert_eq!(autocomplete(&todo, "task +not-exist"), None);
    }
}
