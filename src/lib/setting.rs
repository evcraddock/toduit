use std::env;

pub fn get_root_folder() -> String {
    env::var("TODUIT_ROOT_FOLDER").expect("root folder variable not set")
}

pub fn get_project_folder() -> String {
    env::var("TODUIT_PROJECT_FOLDER").expect("project folder variable not set")
}

pub fn get_project_folder_name() -> String {
    env::var("TODUIT_PROJECT_FOLDER_NAME").expect("project folder name variable not set")
}

pub fn get_journal_folder() -> String {
    env::var("TODUIT_JOURNAL_FOLDER").expect("journal folder variable not set")
}

pub fn get_review_folder() -> String {
    env::var("TODUIT_REVIEW_FOLDER").expect("review folder variable not set")
}

pub fn get_todo_list() -> String {
    env::var("TODUIT_TODO_LISTS").expect("todo lists variable not set")
}

pub fn get_reminder_file() -> String {
    env::var("TODUIT_REMINDER_FILE").expect("reminder file variable not set")
}
