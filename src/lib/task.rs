extern crate chrono;
extern crate serde;
extern crate serde_yaml;

use chrono::prelude::*;
use chrono::DateTime;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind};
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Result;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::util::date_format;

#[derive(Debug)]
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

    pub fn get_by_id_or_name(task: &str, project_folder: &str, new_only: bool) -> Result<Task> {
        
        for entry in WalkDir::new(project_folder)
            .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok()) {
                    let f_path = entry.path().to_str().unwrap();

                    if f_path.ends_with(".md") {
                        if new_only && !f_path.contains("/new/") {
                            continue;
                        }

                        let f_task = Task::get(f_path)?;
                        if f_task.id == task || f_task.task_name == task {
                            return Ok(f_task);
                        }
                    }
        }

        Err(Error::new(ErrorKind::NotFound, "not found"))
    }

    pub fn year_turnover(root_folder: &str, old_year: &str, new_year: &str) -> Result<()> {
        let project_root_folder = format!("{}/Projects/{}", root_folder, old_year);
        for entry in WalkDir::new(project_root_folder)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
                let f_path = entry.path().to_str().unwrap();
                if f_path.ends_with(".md") && f_path.contains("/new/") {
                    let new_task_path = f_path.replace(&old_year, &new_year);
                    let new_path = Task::get_new_folder(&new_task_path);
                    if !Path::new(&new_path).exists() {
                        match fs::create_dir_all(&new_path){
                            Ok(v) => v,
                            Err(e) => eprintln!("could not create folder {} with error {}", new_path, e),
                        }
                    }

                    match fs::rename(f_path, &new_task_path) {
                        Ok(v) => v,
                        Err(e) => eprintln!("could not rename {} with error {}", f_path, e),
                    };

                    let mut f_task = Task::get(&new_task_path)?;
                    f_task.path = f_task.path.replace(&old_year, &new_year);
                    f_task.save(&root_folder).expect("could not save task");
                }
            }

        Ok(())

    }

    pub fn change_project(task_name: &str, project_root_folder: &str, old_project: &str, new_project: &str) -> Result<()> {
        let task_path = format!("{}/{}/new/{}.md", project_root_folder, old_project, task_name);
        
        if !Path::new(&task_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "task file not found"));
        }
    
        let new_path = task_path.replace(&old_project, new_project);
        if !Path::new(&Task::get_new_folder(&new_path)).exists() {
            fs::create_dir_all(&new_path)?;
        }

        // println!("old path: {}\nnew path: {}\n", task_path, new_path);           
        fs::rename(task_path, new_path).expect("could not rename the file");
        Ok(())
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
                ymllink = dest.replace("..", &taskfolder).replace("%20", " ");
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

    pub fn save(self, root_folder: &str) -> Result<()> {
        let mut task = self;
        if task.id == "" || task.id == "~" {
            task.id = Uuid::new_v4().to_string();
        }

        let ymltask = serde_yaml::to_string(&task).unwrap();
        
        let task_path = Task::get_new_folder(&task.path);
        let file_path = format!("{}/{}/new/{}.md", root_folder, task_path, task.task_name);

        if !Path::new(&file_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "task file not found"));
        };

        let data = fs::read_to_string(&file_path).expect("unable to read file");
        let contents: Vec<&str> = data.split("---").collect();

        let mut task_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .expect("could not open task file");
    
        task_file.write_all(format!("{} \n---", ymltask).as_bytes())?;
        task_file.write_all(contents[2].as_bytes())?;
        
        task_file.sync_data()?;

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

    fn get_new_folder(task_path: &str) -> String {
        let mut entries: Vec<&str> = task_path.split_terminator("/").collect();
        entries.remove(entries.len() -1);

        entries.join("/")
    }
}

pub fn update_project(project_folder: &str, old_project: &str, new_project: &str) -> Result<()> {
   for entry in WalkDir::new(project_folder)
       .follow_links(true)
           .into_iter()
           .filter_map(|e| e.ok()) {
               let f_path = entry.path().to_str().unwrap();
               if f_path.ends_with(".md") && f_path.contains("/new/") {
                   let n_path = f_path.replace(old_project, new_project);
                   println!("old path: {}\nnew path: {}\n", f_path, n_path);           
               }
   }

   Ok(())
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

