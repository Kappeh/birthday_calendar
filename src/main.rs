#![forbid(unsafe_code)]

use std::{path::PathBuf, ops::Add, borrow::Cow};

use anyhow::Result;
use chrono::{NaiveDate, Days, Datelike};
use clap::Parser;
use ics::{ICalendar, Event, properties::{DtEnd, DtStart, Summary, Status}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, env)]
    product_id: String,
    #[arg(short, long, env)]
    in_file: PathBuf,
    #[arg(short, long, env)]
    out_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    birthday: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
struct InFile {
    people: Vec<Person>,
}

fn all_day_event<'a, S>(name: S, day: NaiveDate) -> Event<'a>
where
    S: Into<Cow<'a, str>>,
{
    let uid = Uuid::new_v4().to_string();
    let start = day.format("%Y%m%d").to_string();
    let end = day.add(Days::new(1)).format("%Y%m%d").to_string();

    let mut event = Event::new(uid, start.clone());
    event.push(Summary::new(name));
    event.push(DtStart::new(start));
    event.push(DtEnd::new(end));
    event.push(Status::confirmed());

    event
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();
    let in_file_content = std::fs::read_to_string(&args.in_file)?;
    let in_file: InFile = serde_yaml::from_str(&in_file_content)?;

    let today = chrono::offset::Local::now().date_naive();
    let generate_upper = today.add(Days::new(365));

    let mut calendar = ICalendar::new("2.0", args.product_id);

    for person in in_file.people.iter() {
        let birth_year = person.birthday.year_ce().1 as i32;

        for age in 0.. {
            let birthday = match person.birthday.with_year(birth_year + age) {
                Some(date) => date,
                None => continue,
            };
            if birthday > generate_upper {
                break;
            }
            let ordinal_indicator = match age % 10 {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            };
            let event_name = format!("{}'s {}{} Birthday", person.name, age, ordinal_indicator);
            calendar.add_event(all_day_event(event_name, birthday));
        }
    }

    calendar.save_file(args.out_file)?;

    Ok(())
}
