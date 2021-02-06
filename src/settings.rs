extern crate config;
extern crate chrono;

use std::env;
use std::fs;
use std::fs::File;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use chrono::prelude::*;
use chrono::DateTime;


pub struct Settings {
    config: config::Config,
}

impl Settings {
    pub fn new() -> Settings {
        let settings_path = get_settings_path().unwrap();

        let mut conf = config::Config::default();
        conf.merge(config::File::from(settings_path)).unwrap();

        let new_settings = Settings { config: conf };
        new_settings.set_environment_variables().expect("could not create env variables");

        new_settings
    }

    fn set_environment_variables(&self) -> Result<()> {
    
        let date = Local::now();

        self.set_environment_variable_setting("root-folder", "TODUIT_ROOT_FOLDER");
        self.set_environment_variable_setting("project-folder-name", "TODUIT_PROJECT_FOLDER_NAME");
        self.set_environment_variable_setting("journal-folder-name", "TODUIT_JOURNAL_FOLDER_NAME");
        self.set_environment_variable_setting("review-folder-name", "TODUIT_REVIEW_FOLDER_NAME");
        self.set_journal_folder(&date);
        self.set_review_folder(&date);
        self.set_project_folder(&date);
        
        Ok(())
    }

    fn set_environment_variable_setting(&self, key: &str, env_name: &str) {
        let value = self.get_setting(key);
        if value != "" {
            env::set_var(env_name, value);
        }
    }

    fn get_setting(&self, name: &str) -> String {
        self.config.get_str(name).unwrap()
    }
    
    fn set_journal_folder(&self, date: &DateTime<Local>) {
        let strmonth = date.format("%m - %B");
        let folderpath = format!("{}/{}/{}/{}", get_root_folder(), get_journal_folder_name(), date.year(), strmonth);
        let file_exists = Path::new(&folderpath).exists();

        if !file_exists {
            fs::create_dir_all(&folderpath).expect("could not create path");
        }   

        env::set_var("TODUIT_JOURNAL_FOLDER", folderpath);
    }

    fn set_review_folder(&self, date: &DateTime<Local>) {
        let strmonth = date.format("%m - %B");
        let folderpath = format!("{}/{}/{}/{}", get_root_folder(), get_review_folder_name(), date.year(), strmonth);
        let file_exists = Path::new(&folderpath).exists();

        if !file_exists {
            fs::create_dir_all(&folderpath).expect("could not create review folder");
        }   

        env::set_var("TODUIT_REVIEW_FOLDER", folderpath);
    }

    fn set_project_folder(&self, date: &DateTime<Local>) {
        let folderpath = format!("{}/{}/{}", get_root_folder(), get_project_folder_name(), date.year());
        let file_exists = Path::new(&folderpath).exists();

        if !file_exists {
            fs::create_dir_all(&folderpath).expect("could not create project folder");
        }   

        env::set_var("TODUIT_PROJECT_FOLDER", folderpath);
    }
}

fn get_settings_path() -> Result<PathBuf> {
    let home: PathBuf = match dirs::config_dir() {
        Some(path) => path.join("todo"),
        None => PathBuf::from(""),
    };

    if !home.exists() {
        fs::create_dir_all(&home)?;
        create_settings_file(&home).expect("could not create settings file");
    }

    Ok(home.join("Settings"))
}

fn create_settings_file(dir: &PathBuf) -> Result<()> {
    let mut settingsfile = File::create(&dir.join("Settings.toml"))?;

    let home: PathBuf = match dirs::home_dir() {
        Some(path) => path.join(".local/todo"),
        None => PathBuf::from(""),
    };

    settingsfile.write_all(format!("root-folder = {:?} \n", home).as_bytes())?;
    settingsfile.write_all(b"project-folder-name = 'Projects' \n")?;
    settingsfile.write_all(b"journal-folder-name = 'Journal' \n")?;
    settingsfile.write_all(b"review-folder-name = 'Review' \n")?;

    Ok(())
}

fn get_root_folder() -> String {
    env::var("TODUIT_ROOT_FOLDER").expect("root folder variable not set")
}

fn get_project_folder_name() -> String {
    env::var("TODUIT_PROJECT_FOLDER_NAME").expect("project folder name variable not set")
}

fn get_journal_folder_name() -> String {
    env::var("TODUIT_JOURNAL_FOLDER_NAME").expect("journal folder variable not set")
}

fn get_review_folder_name() -> String {
    env::var("TODUIT_REVIEW_FOLDER_NAME").expect("review folder variable not set")
}


