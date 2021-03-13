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

        #[structopt(short = "d", long = "date", default_value = "", help = "MM:DD:YYYY")]
        date: String,

        #[structopt(short = "t", long = "time", default_value = "", help = "HH:MM")]
        time: String,

        #[structopt(short = "n", long = "notice", default_value = "")]
        notice: u32,

        #[structopt(short = "p", long = "project", default_value = "General")]
        project: String,
    },
    Add {
        task_name: String,
        list_name: String,

        #[structopt(short = "p", long = "project", default_value = "")]
        project: String,
    },
    Finish {
        task_name: String,

        #[structopt(short = "p", long = "project", default_value = "")]
        project: String,
    },
    List {
        list_name: String,
    },
    AddJournal, 
    Unlist {
        task_name: String,

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
    Settings::new();

    match args.action {
        Action::Create {
            task_name,
            description,
            year,
            date,
            time,
            notice,
            project
        } => {
            let project_year = get_project_year(&year);
            let task = Task::new(
                &task_name,
                &project,
                &project_year,
            );

            task.add(&description).expect("could not add task");

            if date != "" || time != "" {
                let mut month  = "";
                let mut day = "";
                let mut ryear = "";
                let mut rtime = "";
         
                if date != "" {
                    let date_val = date.split(":").collect::<Vec<&str>>();
                    if date_val[0] != "00" {
                        month = date_val[0];
                    }
                    if date_val[1] != "00" {
                        day = date_val[1];
                    }
                    if date_val[2] != "00" {
                        ryear = date_val[2];
                    }
                }

                if time != "" {
                    rtime = &time;
                }

                task.set_reminder(&month, &day, &ryear, &rtime, &notice).
                    expect("could not set reminder");
            }
        }
        Action::List { list_name } => {
            let task_list = TaskList::get(&list_name);
            let tasks = task_list.get_tasks()
                .expect("could not get task list");

            for task in tasks {
                println!("{} - {}", task.project, task.task_name);
            }

        }
        Action::Add {
            task_name,
            list_name,
            project
        } => {
            let task = Task::get_by_id_or_name(&task_name, false, &project)
                .expect("could not find task");
            let list = TaskList::get(&list_name);
            list.add(task).expect("could not list task");
        }
        Action::Finish {
            task_name,
            project
        } => {
            let task = Task::get_by_id_or_name(&task_name, false, &project)
                .expect("could not find task");
            
            task.finish().expect("could not finish");
        }
        Action::AddJournal => {
            let journal = Journal::new("Journal", "My Thoughts Today").expect("could not find journal path");

            if journal.create().unwrap() {
                let task_list = TaskList::get("Today");
                let tasks = task_list.get_tasks()
                    .expect("could not get task list");

                journal.add_tasks_to_journal(tasks);
            }
        }
        Action::Unlist {
            task_name,
            project
        }=> {
            let task = Task::get_by_id_or_name(&task_name, false, &project)
                .expect("could not find task");

            Task::add_comment(&task, "Unlisted", false).expect("could not add comment");
            Task::move_to_new_folder(&task);
            toduitl::task_list::remove_from_lists(&task_name, "");
        }
        Action::ChangeProject {
            task,
            new_project,
            project
        } => {
            let c_task = Task::get_by_id_or_name(&task, true, &project)
                .expect("the task is either already complete or cannot be found");

            Task::change_project(&c_task, &new_project)
                .expect("could not change the project path");
        }
        Action::TurnoverYear {
            old_year,
            new_year,
        } => {
            Task::year_turnover(&old_year, &new_year).expect("year turnover failed");
        }
        Action::Review {
            project
        } => {
            let tasks = Task::get_all(true, &project).unwrap();
            Task::create_review(tasks).expect("could not create review file");
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
