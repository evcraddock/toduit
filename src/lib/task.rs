extern crate chrono;
extern crate serde;
extern crate serde_yaml;

use chrono::prelude::*;
use chrono::DateTime;
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
use crate::task_list;
use crate::reminder::Reminder;

#[derive(Debug)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_name: String,
    pub project: String,
    pub path: String,

    #[serde(with = "date_format")]
    pub created: DateTime<Local>,
    #[serde(with = "date_format")]
    pub updated: DateTime<Local>,

    pub exclude_from_journal: Option<bool>,
    pub exclude_from_logging: Option<bool>,

    pub remind: Option<Reminder>,
}

impl Task {
    pub fn new(task_name: &str, project: &str, year: &i32) -> Task {
        let created = Local::now();
        let task_path = format!(
            "{}/{}/{}/{}.md",
            crate::setting::get_project_folder_name(),
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
            updated: created,
            exclude_from_journal: None,
            exclude_from_logging: Some(false),
            remind: None
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

        if ymltask.find("updated:").is_none() {
             let today = Local::now();
             let todaystr = format!(
                "{:02}-{:02}-{:02} {:02}:{:02}:{:02}",
                today.year(),
                today.month(),
                today.day(),
                00,
                00, 
                00               
             );

             ymltask.push_str(&format!("updated: \"{}\"", todaystr));
        }

        let task: Task = serde_yaml::from_str(&ymltask).expect("could not deserialize");

        Ok(task)
    }

    pub fn get_by_id_or_name(task: &str, new_only: bool, project: &str) -> Result<Task> {
        let project_folder = format!(
            "{}/{}",
            crate::setting::get_project_folder(),
            if project != "" { project } else { "" }
        );

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

    pub fn get_all(new_only: bool, project: &str) -> Result<Vec<Task>> {
        let mut task_list: Vec<Task> = Vec::new();
        let project_folder = format!(
            "{}/{}",
            crate::setting::get_project_folder(),
            if project != "" { project } else { "" }
        );
            
        for entry in WalkDir::new(&project_folder)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
                let f_path = entry.path().to_str().unwrap();
                if f_path.ends_with(".md") {
                    if new_only && !f_path.contains("/new/") {
                        continue;
                    }
                    
                    match Task::get(f_path) {
                        Ok(v) => task_list.push(v),
                        Err(e) => eprintln!("could not find task {} with error {}", f_path, e),
                    }
                }
            }

        Ok(task_list)
    }

    pub fn is_excluded(&self) -> bool {
        match &self.exclude_from_journal {
            Some(x) => if *x { return true },
            None => ()
        };

        match &self.exclude_from_logging {
            Some(x) => if *x { return true },
            None => ()
        };

        false
    }

    pub fn year_turnover(old_year: &str, new_year: &str) -> Result<()> {
        let root_folder = crate::setting::get_root_folder();
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
                    f_task.save().expect("could not save task");
                }
            }

        Ok(())

    }

    pub fn change_project(&self, new_project: &str) -> Result<()> {
        let old_project = &self.project;
        Task::add_comment(&self, &format!("Project changed to {}", new_project), true)?;
        let project_root_folder = crate::setting::get_project_folder();
        let task_path = format!("{}/{}/new/{}.md", project_root_folder, old_project, &self.task_name);
        
        if !Path::new(&task_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "task file not found"));
        }
    
        let new_path = task_path.replace(old_project, new_project);
        let new_folder = &Task::get_new_folder(&new_path);
        if !Path::new(new_folder).exists() {
            fs::create_dir_all(&new_folder).expect("could not create folder");
        }

        fs::rename(task_path, &new_path).expect("could not rename the file"); 

        let mut task = Task::get(&new_path).expect("could not load task").clone();
        task.project = new_project.to_string();
        task.path = task.path.replace(old_project, new_project);
        task.save().expect("could not save task");

        Ok(())
    }

    pub fn add(&self, description: &str) -> Result<()> {
        let ymltask = serde_yaml::to_string(&self).unwrap();
        let project_folder = crate::setting::get_project_folder();
        let folder_path = get_taskfolder(&project_folder, &self.project, true).unwrap();

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
        taskfile.write_all(format!("##### {} \nTask Created\n\n", today).as_bytes())?;
        if description != "" {
            taskfile.write_all(format!("\n\n[link]({})", description).as_bytes())?;
        }

        taskfile.sync_data()?;
        Ok(())
    }

    pub fn save(self) -> Result<()> {
        let mut task = self;
        if task.id == "" || task.id == "~" {
            task.id = Uuid::new_v4().to_string();
        }

        task.updated = Local::now();
        let ymltask = serde_yaml::to_string(&task).unwrap();
        let task_path = Task::get_new_folder(&task.path);
        let file_path = format!("{}/{}.md", task_path, task.task_name);

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

    pub fn finish(&self, comment: &str) -> Result<()> {
        match Task::add_comment(&self, comment, false) {
            Ok(_o) => (),
            Err(_e) => Task::add_comment(&self, comment, true).expect("could not add comment")
        }
        
        task_list::remove_from_lists(&self.task_name, "none");

        Ok(())
    }

    pub fn set_reminder(&self, month: &str, day: &str, year: &str, time: &str, notice: &u32) -> Result<()> {
        let mut task = self.clone();
        let remind = Reminder::new(month, day, year, time, notice);
        let new_remind = remind.create(&task.task_name);

        match new_remind {
            Ok(_t) => (),
            Err(e) => return Err(e)
        };

        task.remind = Some(remind);
        task.save()
    }

    pub fn add_comment(&self, comment: &str, is_new: bool) -> Result<()> {
        if self.is_excluded() {
            return Ok(());
        }

        let file_path = match is_new {
            false => format!("{}/{}",
                        crate::setting::get_root_folder(),
                        &self.path
                     ),
            true => format!("{}/{}.md",
                        Task::get_new_folder(&self.path),
                        &self.task_name
                    )
        };
        
        if !Path::new(&file_path).exists() {
            return Err(Error::new(ErrorKind::NotFound, "task file not found"));
        };
        
        let data;
        match fs::read_to_string(&file_path) {
            Ok(d) => data = d,
            Err(e) => return Err(e)
        }

        let mut task_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file_path)
            .expect("could not open task file");
        
        let updated = Local::now();
        let (is_pm, hour) = updated.hour12();
        let updated_str = format!(
            "{:02}/{:02}/{:02} {:02}:{:02} {}",
            updated.month(),
            updated.day(),
            updated.year(),
            hour,
            updated.minute(),
            if is_pm { "PM" } else { "AM" }
        );

        let contents: Vec<&str> = data.split("---\n").collect();
        let ymlvalue = contents[1]
            .trim_start_matches('\n')
            .trim_end_matches('\n');


        task_file.write_all(format!("---\n{}\n---\n", ymlvalue).as_bytes())?;
        task_file.write_all(format!("##### {} \n{}\n\n", updated_str, comment).as_bytes())?;
        task_file.write_all(contents[2].as_bytes())?;
        task_file.sync_data()?;

        Ok(())
    }

    pub fn change_task_folder(&self) -> Result<()> {
        let root_folder = crate::setting::get_root_folder();
        let mut entries: Vec<&str> = self.path.split_terminator("/").collect();
        entries.remove(entries.len() -1);

        let file_path = format!(
            "{}/{}/{}/{}.md",
            root_folder,
            entries.join("/"),
            "/new/",
            &self.task_name
        );

        let new_path = format!(
            "{}/{}",
            root_folder,
            &self.path
        );

        if Path::new(&file_path).exists() {
            fs::rename(file_path, &new_path).expect("could not rename the file");
        };

        Ok(())
    }

    pub fn move_to_new_folder(&self) {
        let project_folder = crate::setting::get_project_folder();
        let newpath = format!(
            "{}/{}/new/{}.md",
            project_folder,
            self.project,
            self.task_name
        );

        let oldpath = format!(
            "{}/{}/{}.md",
            project_folder,
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

        let project_path = entries.join("/");

        format!("{}/{}/new/", crate::setting::get_root_folder(), project_path)
    }

    pub fn create_review(tasks: Vec<Task>) -> Result<()> {
        let review_folder = crate::setting::get_review_folder();
        let date = Local::now();
        let review_file_path = format!(
            "{}/{:02}-{:02}-{} Review.md",
            review_folder,
            date.month(),
            date.day(),
            date.year()
        );
    
        let mut review_file  = File::create(&review_file_path)?;
        
        review_file.write_all(
            format!(
                "# {:02}/{:02}/{} Review \n\n",
                date.month(),
                date.day(),
                date.year()
            ).as_bytes()
        ).expect("could not write");
    
        review_file.write_all(b"## Tasks \n").expect("could not write");
        let mut current_project = String::new();
    
        for task in tasks {
            
            if task.project != current_project {
                review_file.write_all(format!("\n#### {} \n", task.project).as_bytes()).expect("could not write");
                current_project = task.project.to_string();
            }

            let new_project = &format!("{}/new", &task.project);
            let new_path = task.path.replace(&task.project, &new_project);
            review_file.write_all(
                format!(
                   "* [{}](../../../{}) \n",
                   task.task_name,
                   new_path,
                ).as_bytes()
            ).expect("failed to write");
    
            review_file.sync_data()?;
        }
    
        Ok(())    
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

pub fn get_task_path_old(task_name: &str, project: &str, project_folder: &str) -> Result<String> {
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


