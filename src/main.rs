extern crate kernel32;
extern crate kernel32x;
extern crate winapi;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate shlex;
extern crate ctrlc;
extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate wol;

use simplelog::{TermLogger, LogLevelFilter};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::{Command, Stdio};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};
use std::sync::Arc;
use std::fs::File;
use std::io::Read;

mod vss;

// const REPO: &str = "rest:http://192.168.0.3:8000/";
// const PASS: &str = "vault";
static STOP: AtomicBool = ATOMIC_BOOL_INIT;

#[derive(Debug, Deserialize)]
struct Config {
  env: Option<HashMap<String,String>>,
  general: GeneralConfig,
}

#[derive(Debug, Deserialize)]
struct GeneralConfig {
  shadow: Option<Vec<char>>,
  commands: Vec<String>,
}

fn to_wstring(str: &str) -> Vec<u16> {
  let v: Vec<u16> = OsStr::new(str).encode_wide().chain(Some(0).into_iter()).collect();
  v
}

#[cfg(debug_assertions)]
pub fn get_exe_dir() -> String {
  return ".".to_owned();
}

#[cfg(not(debug_assertions))]
pub fn get_exe_dir() -> String {
  let mut path: Vec<u16> = vec![0; winapi::MAX_PATH];
  unsafe {
    kernel32x::GetModuleFileNameW(0 as winapi::HMODULE,
                                  path.as_mut_ptr(),
                                  winapi::MAX_PATH as u32);
  }
  let path = String::from_utf16(path.as_slice()).unwrap();
  let path = Path::new(&path);
  let parent = path.parent().unwrap();
  parent.to_str().unwrap().to_owned()
}

fn extract_drive_letters<T: AsRef<str>>(sources: &[T]) -> Vec<char> {
  let mut drives = HashSet::new();
  for source in sources {
    let source = source.as_ref();
    if source.len() > 1 && source.chars().nth(1).unwrap() == ':' {
      drives.insert(source.chars().nth(0).unwrap());
    }
  }

  drives.into_iter().collect()
}

fn run<T: Into<String>>(env: &HashMap<String,String>, cmd: T) {
  if STOP.load(Ordering::SeqCst) {
    return;
  }

  let cmd = cmd.into();
  println!("{}", cmd);
  let mut args = shlex::split(&cmd).unwrap();
  let mut cmd = Command::new(args.remove(0));
  cmd.args(args);
  for (k, v) in env {
    cmd.env(k, v);
  }

  trace!("{:?}", cmd);
  
  cmd.stdout(Stdio::inherit())
  .stderr(Stdio::inherit())
  .output().unwrap();
}

fn main() {
  TermLogger::init(LogLevelFilter::Trace).unwrap();

//   wol::send(
//   vec![0x0C,0xC4,0x7A,0xC5,0x7F,0xDF],
//   "255.255.255.255:9",
//   "0.0.0.0:0"
// );

  ctrlc::set_handler(move || {
    STOP.store(true, Ordering::SeqCst);
  }).expect("Error setting Ctrl-C handler");

  let mut file = File::open("config.toml").unwrap();
  let mut contents = String::new();
  file.read_to_string(&mut contents).unwrap();
  let cfg: Config = toml::from_str(&contents).unwrap();
 
  let mut v = vss::Vss::new();
  v.create(cfg.general.shadow).unwrap(); // destroyed on drop
  let mapped_drives = v.get_mapped_drives();

  for cmd in cfg.general.commands {
    let mut c = cmd;
    for (k, v) in &mapped_drives {
      c = c.replace(&format!("${{{}}}", k), &v.to_string());
    }
    
    run(&cfg.env.clone().unwrap_or(HashMap::new()), c);
  }
}
