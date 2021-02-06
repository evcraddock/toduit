extern crate chrono;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{ BufReader, Result, Write };
use std::path::{ Path, PathBuf};

use pulldown_cmark::{Event, Options, Parser, Tag};
use walkdir::WalkDir;

use crate::task::Task;
use crate::journal::*;

pub struct TaskList {
    pub name: String,
    pub path: String,
}

impl TaskList {
    pub fn get(name: &str) -> TaskList {
        TaskList {
            name: name.to_string(),
            path: format!("{}/{}", crate::setting::get_root_folder(), name),
        }
    }

    pub fn add(&self, task: &Task) -> Result<()> {
        if Path::new(&self.path).exists() {
            let listpath = format!("{}/{}.md", &self.path, &task.task_name);
            let mut listfile = File::create(&listpath)?;
            let task_link = format!("[{}](../{})", &task.task_name, &task.path);
            listfile.write_all(task_link.as_bytes())?;
            Task::change_task_folder(&task).expect("could not change task folder");
            remove_from_lists(&task.task_name, &self.name);
            
            if &self.name == "Today" {
                let journal = Journal::new("Current", "Journal").expect("could not find journal");
                journal.add_task_to_journal(&task);
            }
        }        

        Ok(())
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let mut tasks_list: Vec<Task> = Vec::new();
        for entry in WalkDir::new(&self.path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
                let list_path = entry.path().to_str().unwrap();
                
                if !list_path.ends_with(".md") {
                    continue;
                }
                
                let p = PathBuf::from(&list_path);
                let task_file = File::open(p).expect("could not open file");
                let mut contents = String::new();
                let mut buf_reader = BufReader::new(task_file);

                buf_reader.read_to_string(&mut contents)?;
                // Markdown parser is unable to read links
                // with spaces in the path
                // Replacing with encoded space to be removed
                // before opening the file later
                contents = contents.replace(" ", "%20");

                let options = Options::empty();
                let parser = Parser::new_ext(&contents, options);

                let ps: Vec<Event> = parser.collect();
                for p in &ps {
                    if let Event::Start(Tag::Link(_, dest, _)) = p {
                        let entries: Vec<&str> = dest.split_terminator("/").collect();
                        let task_name = entries[entries.len() -1].replace("%20", " ").replace(".md", "");
                        let etask = Task::get_by_id_or_name(&task_name, false, "").expect("could not get task");
                        tasks_list.push(etask);
                    }
                }
        }

        Ok(tasks_list)
    }
}

pub fn remove_from_lists(task_name: &str, excluded_list: &str) {
    // TODO: this list should be pulled from configuration
    let root_folder = crate::setting::get_root_folder();
    let valid_list = ["Queued", "Today", "Waiting"];
    for list in valid_list.iter() {
        if list.to_string() != excluded_list {
            let filepath = format!("{}/{}/{}.md", root_folder, list.to_string(), task_name);
            if Path::new(&filepath).exists() {
                fs::remove_file(filepath).expect("could not remove from list");
            }
        }
    }
}
