extern crate chrono;
mod settings;
mod task_list;

use chrono::prelude::*;
use toduitl::journal::*;
use toduitl::task::*;
use structopt::StructOpt;
use settings::*;
use task_list::*;

#[derive(StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    action: Action,
}

#[derive(StructOpt)]
enum Action {
    Create {
        task_name: String,

        #[structopt(default_value = "")]
        description: String,

        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,

        #[structopt(short = "p", long = "project", default_value = "General")]
        project: String,
    },
    Add {
        task_name: String,
        list_name: String,

        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,

        #[structopt(short = "p", long = "project", default_value = "General")]
        project: String,
    },
    List {
        list_name: String,

        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,
    },
    AddJournal {
        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,
    },
    Unlist {
        task_name: String,

        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,

        #[structopt(short = "p", long = "project", default_value = "General")]
        project: String,
    }
}

fn main() {
    let args = Cli::from_args();
    let settings = Settings::new();

    match args.action {
        Action::Create { task_name, description, year, project } => {
            let project_year = get_project_year(&year);
            let project_folder = settings.get_project_folder_by_year(&project_year);

            let task = Task::new(
                &task_name,
                &project,
                &settings.get_setting("project-folder-name"),
                &project_year,
            );

            task.add(&description, &project_folder)
                .expect("could not add task");
        }
        Action::List { list_name, year } => {
            let tasks = get_tasks(
                &format!("{}/{}", settings.get_setting("root-folder"), list_name),
                &settings.get_project_folder_by_year(&get_project_year(&year))
            );

            for task in tasks {
                println!("{} - {}", task.project, task.task_name);
            }
        }
        Action::Add {
            task_name,
            list_name,
            year,
            project,
        } => {
            let listpath = &format!("{}/{}", settings.get_setting("root-folder"), &list_name);
            let project_year = get_project_year(&year);
            let project_folder = settings.get_project_folder_by_year(&project_year);
            let taskpath = get_task_path(&task_name, &project, &project_folder).unwrap();
            let task = Task::get(&taskpath).unwrap();

            if put_in_list(&listpath, &task).unwrap() {
                task.change_task_folder(&settings.get_project_folder_by_year(&project_year));
                remove_from_lists(&settings.get_setting("root-folder"), &task_name, &list_name);

                let journalfolder = settings.get_journal_folder_by_date(&Local::now()).unwrap();
                // TODO add ability  to add customer header and subheader 
                let journal = Journal::new("Journal", "My Thoughts Today", &journalfolder);

                if list_name == "Today" {
                    journal.add_task_to_journal(&task);
                }
            } else {
                println!("Could not add the item to the list");
            }
        }
        Action::AddJournal { year } => {
            let projectfolder = settings.get_project_folder_by_year(&get_project_year(&year));
            let journalfolder = settings.get_journal_folder_by_date(&Local::now()).unwrap();
            let journal = Journal::new("Journal", "My Thoughts Today", &journalfolder);

            journal.create().expect("could not create journal");

            let tasks = get_tasks(
                &format!("{}/Today", settings.get_setting("root-folder")),
                &projectfolder,
            );
            journal.add_tasks_to_journal(tasks);
        }
        Action::Unlist {
            task_name,
            year,
            project
        }=> {
            let project_year = get_project_year(&year);
            let project_folder = settings.get_project_folder_by_year(&project_year);
            let taskpath = get_task_path(&task_name, &project, &project_folder).unwrap();
            let task = Task::get(&taskpath).unwrap();
            
            task.remove_from_task_folder(&project_folder);
            remove_from_lists(&settings.get_setting("root-folder"), &task_name, "");
        }
    }
}

fn get_project_year(year: &str) -> i32 {
    let mut project_year = Local::now().year();
    if year != "" {
        project_year = year.parse::<i32>().unwrap();
    }

    project_year
}
