extern crate chrono;
mod settings;

use chrono::prelude::*;
use toduitl::journal::*;
use toduitl::task::*;
use structopt::StructOpt;
use settings::*;
use toduitl::task_list::*;

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

        #[structopt(short = "p", long = "project", default_value = "")]
        project: String,
    },
    List {
        list_name: String,

        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,
    },
    AddJournal, 
    Unlist {
        task_name: String,

        #[structopt(short = "y", long = "year", default_value = "")]
        year: String,

        #[structopt(short = "p", long = "project", default_value = "")]
        project: String,
    },
    ChangeProject {
        task: String,
        new_project: String,
        
        #[structopt(short = "p", long = "project", default_value = "")]
        project: String,
    },
    TurnoverYear {
        old_year: String,
        new_year: String,
    },
    Review {
        #[structopt(short = "p", long = "project", default_value = "")]
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
            let task_list = TaskList::get(&list_name, &settings.get_setting("root-folder"));
            let tasks = task_list.get_tasks(&settings.get_project_folder_by_year(&get_project_year(&year)))
                .expect("could not get task list");

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
            let project_folder = format!(
                "{}/{}",
                settings.get_project_folder_by_year(&get_project_year(&year)),
                project
            );

            let task = Task::get_by_id_or_name(&task_name, &project_folder, false)
                .expect("could not find task");
            let list = TaskList::get(&list_name, &settings.get_setting("root-folder"));
            list.add(&task).expect("could not list task");
        }
        Action::AddJournal => {
            let journalfolder = settings.get_journal_folder_by_date(&Local::now()).unwrap();
            let journal = Journal::new("Journal", "My Thoughts Today", &journalfolder);

            journal.create().expect("could not create journal");
            let task_list = TaskList::get("Today", &settings.get_setting("root-folder"));
            let tasks = task_list.get_tasks(&settings.get_project_folder_by_year(&Local::now().year()))
                .expect("could not get task list");

            journal.add_tasks_to_journal(tasks);
        }
        Action::Unlist {
            task_name,
            year,
            project
        }=> {
            let project_folder = format!(
                "{}/{}",
                settings.get_project_folder_by_year(&get_project_year(&year)),
                project
            );
            let task = Task::get_by_id_or_name(&task_name, &project_folder, false)
                .expect("could not find task");
            
            task.remove_from_task_folder(&project_folder);
            toduitl::task_list::remove_from_lists(&settings.get_setting("root-folder"), &task_name, "");
        }
        Action::ChangeProject {
            task,
            new_project,
            project
        } => {
            let date = Local::now();
            let project_root_folder = settings.get_project_folder_by_year(&date.year());
            let project_folder = format!(
                "{}/{}",
                project_root_folder,
                project,
            );


            let c_task = Task::get_by_id_or_name(&task, &project_folder, true)
                .expect("the task is either already complete or cannot be found");

            Task::change_project(&task, &project_root_folder, &c_task.project, &new_project)
                .expect("could not change the project path");

            let mut new_task = c_task;

            new_task.project = new_project.to_string();
            new_task.path = format!(
                "{}/{}/{}/{}.md", 
                settings.get_setting("project-folder-name"),
                date.year(),
                new_project,
                new_task.task_name
            );

            new_task.save(&settings.get_setting("root-folder"))
                .expect("could not save task");
        }
        Action::TurnoverYear {
            old_year,
            new_year,
        } => {
            Task::year_turnover(&settings.get_setting("root-folder"), &old_year, &new_year).expect("year turnover failed");
        }
        Action::Review {
            project
        } => {
            let date = Local::now();
            let project_root_folder = settings.get_project_folder_by_year(&date.year());
            let project_folder = format!(
                "{}/{}",
                project_root_folder,
                project,
            );

              
            let review_folder = settings.get_review_folder_by_date(&date).unwrap();
            let tasks = Task::get_all(&project_folder, true).unwrap();

            Task::create_review(tasks, &review_folder).expect("could not create review file");
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
