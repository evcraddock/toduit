extern crate chrono;

use chrono::prelude::*;
use chrono::DateTime;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Result;
use std::io::Write;
use std::path::Path;

use crate::task::Task;

pub struct Journal {
    title: String,
    subheader: String,
    pub journal_path: String,
    pub created: DateTime<Local>,
}

impl Journal {
    pub fn new(title: &str, subheader: &str, journal_folder: &str) -> Journal {
        let created = Local::now();
        let filepath = format!(
            "{}/{:02}-{:02}-{} Journal.md",
            journal_folder,
            created.month(),
            created.day(),
            created.year()
        );

        Journal {
            title: title.to_string(),
            subheader: subheader.to_string(),
            created,
            journal_path: filepath,
        }
    }

    pub fn get(date: DateTime<Local>, journal_folder: &str) -> Result<Journal> {
        let strmonth = date.format("%m - %B");
        let file_path = format!(
            "{}/{}/{}/{:02}-{:02}-{} Journal.md",
            journal_folder,
            date.year(),
            strmonth,
            date.month(),
            date.day(),
            date.year()
        );

        let new_journal = Journal {
            title: String::new(),
            subheader: String::new(),
            created: date,
            journal_path: file_path,
        };

        Ok(new_journal)
    }

    pub fn create(&self) -> Result<()> {
        let mut journalfile = File::create(&self.journal_path)?;

        journalfile.write_all(
            format!(
                "# {:02}/{:02}/{} {} \n\n",
                self.created.month(),
                self.created.day(),
                self.created.year(),
                self.title
            )
            .as_bytes(),
        )?;
        journalfile.write_all(format!("## {}\n\n", self.subheader).as_bytes())?;

        journalfile.sync_data()?;
        Ok(())
    }

    pub fn add_link_to_journal(&self, title: &str, link: &str) -> Result<()> {
        if !Path::new(&self.journal_path).exists() {
            self.create().expect("could not create journal")
        };

        let mut journalfile = OpenOptions::new()
            .append(true)
            .open(&self.journal_path)
            .expect("could not append to journal");

        journalfile
                .write_all(format!("* [{}]({})\n", title, link).as_bytes())
                .expect("could not write to journal");

        Ok(())

    }

    pub fn add_task_to_journal(&self, task: &Task) {
        if !Path::new(&self.journal_path).exists() {
            self.create().expect("could not create journal")
        };

        let mut journalfile = OpenOptions::new()
            .append(true)
            .open(&self.journal_path)
            .expect("could not append to journal");

        if !task.exclude_from_journal {
            let link = format!("../../../{}", task.path);
            journalfile
                .write_all(format!("* [{}]({})\n", task.task_name, link).as_bytes())
                .expect("could not write to journal");
        }
    }

    pub fn add_tasks_to_journal(&self, tasks: Vec<Task>) {
        let mut journalfile = OpenOptions::new()
            .append(true)
            .open(&self.journal_path)
            .expect("could not open journal");

        journalfile
            .write_all(String::from("## Tasks \n").as_bytes())
            .expect("could not write to journal");

        for task in tasks {
            if !task.exclude_from_journal {
                let link = format!("../../../{}", task.path);
                journalfile
                    .write_all(format!("* [{}]({})\n", task.task_name, link).as_bytes())
                    .expect("could not write to journal");
            }
        }

        journalfile.sync_data().expect("could not sync data");
    }
}

