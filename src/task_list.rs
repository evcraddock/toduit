use std::fs;
use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use toduitl::task::Task;

pub fn put_in_list(list_path: &str, task: &Task) -> Result<bool> {
    let mut addedtolist = false;
    if Path::new(&list_path).exists() {
        let listpath = format!("{}/{}.md", &list_path, &task.task_name);
        let mut listfile = File::create(&listpath)?;
        let task_link = format!("[{}](../{})", &task.task_name, &task.path);
        listfile.write_all(task_link.as_bytes())?;
        addedtolist = true;
    }

    Ok(addedtolist)
}

pub fn remove_from_lists(list_folder: &str, task_name: &str, excluded_list: &str) {
    // TODO: this list should be pulled from configuration
    let valid_list = ["Queued", "Today", "Waiting"];
    for list in valid_list.iter() {
        if list.to_string() != excluded_list {
            let filepath = format!("{}/{}/{}.md", list_folder, list.to_string(), task_name);
            if Path::new(&filepath).exists() {
                fs::remove_file(filepath).expect("could not remove from list");
            }
        }
    }
}
