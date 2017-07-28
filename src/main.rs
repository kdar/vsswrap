extern crate kernel32;
extern crate kernel32x;
extern crate winapi;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate clap;

use simplelog::{WriteLogger, Config, LogLevelFilter};
use std::process::{Command, Stdio};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::io::stderr;
use clap::{App, AppSettings, Arg, SubCommand};

mod vss;


fn main() {
  //TermLogger::init(LogLevelFilter::Trace).unwrap();
  WriteLogger::init(LogLevelFilter::Trace, Config::default(), stderr()).unwrap();

  let mut v = vss::Vss::new();

  let matches = App::new("vsswrap")
                        .version("1.0")
                        .author("Kevin Darlington <kevin@outroot.com>")
                        .about("Simple wrapper around VSS to make it easier to use.")
                        .subcommand(SubCommand::with_name("create")
                                                .about("creates the shadow drives")
                                                .setting(AppSettings::ArgRequiredElseHelp)
                                                .arg(Arg::with_name("drive").multiple(true)))
                        .subcommand(SubCommand::with_name("delete")
                                                .about("deletes the shadow drives"))
                        .get_matches();
 
   match matches.subcommand() {
      ("create", Some(create_matches)) => {        
        let drives = create_matches.values_of("drive").unwrap().map(|x| x.chars().nth(0).unwrap()).collect::<Vec<_>>();
        v.create(drives).unwrap();
        for (key, value) in v.get_mapped_drives() {
          println!("{} {}", key.to_uppercase().to_string(), value);
        }
      },
      ("delete", Some(delete_matches)) => {
        v.delete();
      },
      ("", None)     => println!("No subcommand was used"),
        _           => unreachable!(),
    }
}
