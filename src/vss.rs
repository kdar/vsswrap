use std::path::Path;
use std::io::{BufRead, BufReader};
use std::error::Error;
use std::process::{Command, exit};
use std::fs::File;
use std::collections::HashMap;
use kernel32;
use kernel32x;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

fn to_wstring(str: &str) -> Vec<u16> {
  let v: Vec<u16> = OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v
}

fn parse_vars<P: AsRef<Path>>(path: P) -> Result<Vec<(String, String)>, Box<Error>> {
  let mut results = vec![];
  let pb = path.as_ref().to_path_buf();
  let file = try!(File::open(pb));
  for line in BufReader::new(file).lines() {
    let line = try!(line);
    if line.starts_with("SET") {
      let afterset = line.chars().skip(4).collect::<String>();
      let v: Vec<&str> = afterset.split("=").collect();
      results.push((v[0].to_owned(), v[1].to_owned()));
    }
  }

  Ok(results)
}

pub struct Vss {
  mapped_drives: HashMap<char, char>,
}

impl Vss {
  pub fn new() -> Vss {
    Vss { mapped_drives: HashMap::new() }
  }

  pub fn get_mapped_drives(&self) -> HashMap<char, char> {
    self.mapped_drives.clone()
  }

  pub fn create(&mut self, drive_letters: Vec<char>) -> Result<(), String> {
    self.delete();

    info!("Grabbing available drive letters...");

    // Find available drive letters we can use.
    let mut available_drives: Vec<char> = vec![];
    let drives = unsafe { kernel32::GetLogicalDrives() };
    for i in 0..26 {
      if ((drives >> i) & 1) == 0 {
        available_drives.push((('A' as u8) + i) as char);
      }
      // println!("{} - {}", (('a' as u8) + i) as char, (drives >> i) & 1);
    }

    if available_drives.len() < drive_letters.len() {
      return Err("Not enough available drive letters to map to shadow drives.".to_owned());
    }

    info!("Creating shadow volumes...");
    let mut cmd = Command::new("vendor\\vshadow64.exe");
    cmd.arg("-p")
      .arg("-nw")
      .arg("-script=vss-vars.cmd");
    for k in &drive_letters {
      cmd.arg(format!("{}:", k));
    }
    let output = cmd.output();
    match output {
      Ok(_) => {}
      Err(e) => {
        error!("{}: {:?}", e, cmd);
        exit(1);
      }
    };

    let vss_vars = parse_vars("vss-vars.cmd").unwrap();
    let mut shadow_devices = vec![];
    for var in vss_vars {
      if var.0.starts_with("SHADOW_DEVICE_") {
        shadow_devices.push(var.1);
      }
    }

    if shadow_devices.len() < drive_letters.len() {
      return Err("Could not create shadow devices for all drives.".to_owned());
    }

    let mut drive_letters = drive_letters.iter();
    let mut available_drives = available_drives.iter();
    let mut shadow_devices = shadow_devices.iter();
    while let Some(drive_letter) = drive_letters.next() {
      let available_drive = available_drives.next().unwrap();
      let shadow_device = shadow_devices.next().unwrap();

      let result = unsafe {
        kernel32x::DefineDosDeviceW(0,
                                    to_wstring(&format!("{}:", available_drive)).as_ptr(),
                                    to_wstring(shadow_device).as_ptr())
      };

      if result != 1 {
        return Err("Could not create drive letter for shadow device.".to_owned());
      }

      info!("Drive \"{}:\" shadowed to \"{}\" -> \"{}:\"",
            drive_letter,
            shadow_device,
            available_drive);

      self.mapped_drives.insert(*drive_letter, *available_drive);
    }

    Ok(())
  }

  pub fn delete(&self) {
    if Path::new("vss-vars.cmd").exists() {
      let vars = parse_vars("vss-vars.cmd").unwrap();
      info!("Removing volume shadow set: \"{}\"", vars[0].1);
      let mut cmd = Command::new("vendor\\vshadow64.exe");
      cmd.arg(format!("-dx={}", vars[0].1));
      match cmd.output() {
        Ok(_) => {}
        Err(e) => {
          error!("{}: {:?}", e, cmd);
          exit(1);
        }
      };
    }
  }
}

