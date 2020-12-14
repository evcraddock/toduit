extern crate chrono;
extern crate serde;
extern crate serde_yaml;

use chrono::prelude::*;
use chrono::DateTime;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::util::date_format;

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_name: String,
    pub project: String,
    pub path: String,

    #[serde(with = "date_format")]
    pub created: DateTime<Local>,
    pub exclude_from_journal: bool,
}

impl Task {
    pub fn new(task_name: &str, project: &str, project_folder_name: &str, year: &i32) -> Task {
        let created = Local::now();
        let task_path = format!(
            "{}/{}/{}/{}.md",
            project_folder_name,
            year,
            project,
            task_name
        );

        Task {
            id: Uuid::new_v4().to_string(),
            task_name: task_name.to_string(),
            project: project.to_string(),
            path: task_path,
            created,
            exclude_from_journal: false,
        }
    }

    pub fn get(filepath: &str) -> Result<Task> {
        let p = PathBuf::from(filepath);
        let newfile = File::open(p)?;

        let mut buf_reader = BufReader::new(newfile);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        let contents = contents.split("---");
        let fm = contents.collect::<Vec<&str>>();
        let mut ymltask = fm[1].to_string();
        if ymltask.find("id:").is_none() {
            ymltask.push_str("id: ");
        }

        let task: Task = serde_yaml::from_str(&ymltask).unwrap();

        Ok(task)
    }

    pub fn get_from_list(listpath: &str, taskfolder: &str) -> Result<Task> {
        let p = PathBuf::from(listpath);

        let taskfile = File::open(p)?;
        let mut contents = String::new();
        let mut ymllink = String::new();
        let mut buf_reader = BufReader::new(taskfile);

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
                ymllink = dest.replace("../Projects", &taskfolder).replace("%20", " ");
            }
        }

        Task::get(&ymllink)
    }

    pub fn add(&self, description: &str, project_folder: &str) -> Result<()> {
        let ymltask = serde_yaml::to_string(&self).unwrap();
        let folder_path = get_taskfolder(project_folder, &self.project, true).unwrap();

        let filepath = format!("{}/{}.md", folder_path, &self.task_name);
        let mut taskfile = File::create(&filepath)?;
        let (is_pm, hour) = self.created.hour12();
        let today = format!(
            "{:02}/{:02}/{:02} {:02}:{:02} {}",
            self.created.month(),
            self.created.day(),
            self.created.year(),
            hour,
            self.created.minute(),
            if is_pm { "PM" } else { "AM" }
        );
        taskfile.write_all(format!("{} \n---\n", ymltask).as_bytes())?;
        taskfile.write_all(format!("# {} \n\n", self.task_name).as_bytes())?;
        taskfile.write_all(format!("##### {} \nTask Created", today).as_bytes())?;
        if description != "" {
            taskfile.write_all(format!("\n\n[link]({})", description).as_bytes())?;
        }

        taskfile.sync_data()?;
        Ok(())
    }

    pub fn change_task_folder(&self, task_folder: &str) {
        let filepath = format!(
            "{}/{}/new/{}.md",
            task_folder,
            self.project,
            self.task_name
        );
        let newpath = format!(
            "{}/{}/{}.md",
            task_folder,
            self.project,
            self.task_name
        );

        println!("new-path: {}", newpath);

        if Path::new(&filepath).exists() {
            fs::rename(filepath, &newpath).expect("could not rename the file");
        }
    }

    pub fn remove_from_task_folder(&self, task_folder: &str) {
        let newpath = format!(
            "{}/{}/new/{}.md",
            task_folder,
            self.project,
            self.task_name
        );

        let oldpath = format!(
            "{}/{}/{}.md",
            task_folder,
            self.project,
            self.task_name
        );

        if Path::new(&oldpath).exists() {
            fs::rename(oldpath, &newpath).expect("could not rename the file");
        }
    }
}

pub fn get_taskfolder(task_folder: &str, project: &str, is_new: bool) -> Result<String> {
    let folderpath = format!(
        "{}/{}/{}",
        task_folder,
        project,
        if is_new { "new" } else { "" }
    );

    fs::create_dir_all(&folderpath)?;

    Ok(folderpath)
}

pub fn get_tasks(listfolder: &str, projectfolder: &str) -> Vec<Task> {
    let mut v: Vec<Task> = Vec::new();
    let files = fs::read_dir(&listfolder).unwrap();

    for file in files {
        let filename = file.unwrap().file_name().into_string().unwrap();
        let task =
            Task::get_from_list(&format!("{}/{}", listfolder, filename), projectfolder).unwrap();
        v.push(task);
    }

    v
}

pub fn get_task_path(task_name: &str, project: &str, project_folder: &str) -> Result<String> {
    let filepath = format!(
        "{}/{}/new/{}.md",
        project_folder,
        project,
        task_name
    );

    let newpath = format!(
        "{}/{}/{}.md",
        project_folder,
        project,
        task_name
    );

    let mut returnpath = newpath;
    if Path::new(&filepath).exists() {
        returnpath = filepath;
    }

    Ok(returnpath)
}

