extern crate chrono;

use chrono::prelude::*;
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
    pub root_path: String,
}

impl TaskList {
    pub fn get(name: &str, root_path: &str) -> TaskList {
        TaskList {
            name: name.to_string(),
            path: format!("{}/{}", root_path, name),
            root_path: root_path.to_string(),
        }
    }

    pub fn add(&self, task: &Task) -> Result<()> {
        if Path::new(&self.path).exists() {
            let listpath = format!("{}/{}.md", &self.path, &task.task_name);
            let mut listfile = File::create(&listpath)?;
            let task_link = format!("[{}](../{})", &task.task_name, &task.path);
            listfile.write_all(task_link.as_bytes())?;
            Task::change_task_folder(&task, &self.root_path).expect("could not change task folder");
            remove_from_lists(&self.root_path, &task.task_name, &self.name);
            
            let journalfolder = format!("{}/Journal", &self.root_path);
            let journal = Journal::get(Local::now(), &journalfolder).expect("could not create journal");

            if &self.name == "Today" {
                journal.add_task_to_journal(&task);
            }
        }        

        Ok(())
    }

    pub fn get_tasks(&self, taskfolder: &str) -> Result<Vec<Task>> {
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
                        let etask = Task::get_by_id_or_name(&task_name, &taskfolder, false).expect("could not get task");
                        tasks_list.push(etask);
                       // ymllink = dest.replace("..", &taskfolder).replace("%20", " ");
                    }
                }
        }

        Ok(tasks_list)
    }
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
