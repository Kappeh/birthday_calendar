#![forbid(unsafe_code)]

use std::{path::PathBuf, ops::Add};

use anyhow::Result;
use chrono::{NaiveDate, Days};
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

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Args::parse();
    let people_yaml = std::fs::read_to_string(&args.in_file)?;
    let config: InFile = serde_yaml::from_str(&people_yaml)?;

    let mut calendar = ICalendar::new("2.0", args.product_id);
    for person in config.people.iter() {
        let start = person.birthday.format("%Y%m%d").to_string();
        let end = person.birthday.add(Days::new(1)).format("%Y%m%d").to_string();

        let uid = Uuid::new_v4().to_string();
        let mut event = Event::new(uid, start.clone());
        event.push(DtStart::new(start));
        event.push(DtEnd::new(end));
        event.push(Summary::new(format!("{}'s Birthday", person.name)));
        event.push(Status::confirmed());

        calendar.add_event(event);
    }

    calendar.save_file(args.out_file)?;

    Ok(())
}
