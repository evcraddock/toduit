extern crate chrono;
extern crate num_traits;

use num_traits::cast::FromPrimitive;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions};
use std::io::Result;
use std::io::Write;

#[derive(Debug)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub month: String,
    pub day: String,
    pub year: String,
    pub time: String,
    pub notice: u32,
}

impl Reminder {

    pub fn new(month: &str, day: &str, year: &str, time: &str, notice: &u32) -> Reminder {
        Reminder {
            month: month.to_string(),
            day: day.to_string(),
            year: year.to_string(),
            time: time.to_string(),
            notice: *notice
        }
    }

    pub fn create(&self, task_name: &str) -> Result<()> {
        let reminder = &self.clone();
        let reminder_file_path = crate::setting::get_reminder_file();
        let mut reminder_file = OpenOptions::new()
            .append(true)
            .open(&reminder_file_path)
            .expect("could not open reminder file");

        let date_value = &self.get_reminder_date();
        let date = match date_value {
            Ok(d) => d,
            Err(_e) => ""
        };

        let reminder_time = if reminder.time != "" { format!("AT {}", reminder.time) } else { "".to_string() };

        let notice = &reminder.notice.to_string();
        let reminder_notice = if reminder.notice > 0 { format!("-{}", notice) } else { "".to_string() };
        let rem_entry = format!("REM {} {} {} MSG %\"{}%\" [t()] \n",
           date,
           reminder_time,
           reminder_notice,
           task_name
        );

        let run_entry = format!("REM {} {} {} RUN ({}) & \n",
            date,
            reminder_time,
            reminder_notice,
            format!("toduit add \"{}\" Today", task_name)
        );

        reminder_file.write_all(rem_entry.as_bytes())?;
        reminder_file.write_all(run_entry.as_bytes())?;
        reminder_file.sync_data()?;

        Ok(())
    }

    fn get_reminder_date(&self) -> Result<String> {
        let reminder = &self.clone();

        let month_value = &reminder.month.parse::<u32>();
        let month = match month_value {
            Ok(m) => m,
            Err(_e) => &0,
        };

        
        let month_name = Month::from_u32(*month);
        let month_string = match month_name {
            Some(m) => m.name(),
            None => "",
        };
       
        let rem_date = format!("{} {} {}", 
            if &reminder.day != "" { &reminder.day } else { "" },
            month_string,
            if &reminder.year != "" { &reminder.year } else { "" }
        );

        Ok(rem_date)
    }
}
