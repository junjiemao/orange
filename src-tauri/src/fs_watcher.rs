extern crate notify;

use crate::idx_store::IDX_STORE;
use crate::utils;
use crate::utils::subs;
use log::{error, info};
use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "macos")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;
use std::path;

use std::sync::mpsc::channel;

pub struct FsWatcher {
  path: String,
}

impl FsWatcher {
  pub fn new(path: String) -> FsWatcher {
    FsWatcher { path }
  }

  pub fn start(&mut self) {
    let (tx, rx) = channel();
    let mut watcher = raw_watcher(tx).unwrap();
    let wt_res = watcher.watch(self.path.as_str(), RecursiveMode::Recursive);
    if wt_res.is_err() {
      error!("{:?}", wt_res.err());
      error!("watch {} err ", self.path);
      return;
    }
    info!("fs watcher started");

    loop {
      match rx.recv() {
        Ok(RawEvent {
          path: Some(path),
          op: Ok(op),
          cookie: _,
        }) => {
          let path_str = path.to_str().unwrap();
          let abs_path = path_str.to_string();
          if path_str.contains("orangecachedata") {
            continue;
          }
          if Op::REMOVE & op == Op::REMOVE {
            IDX_STORE._del(abs_path)
          } else {
            let name: String = utils::path2name(abs_path.clone());
            let name0 = name.clone();
            let ext = utils::file_ext(name0.as_str());
            //如果ext是txt,md,则索引文本内容
            if ext.eq("txt") || ext.eq("md") {
              //let content = std::fs::read_to_string(abs_path.clone());
              let content = "关关雎鸠，在河之洲";
              //if content.is_ok() 
              //if content.is_some(){
                //let content: String = content.unwrap();
                IDX_STORE.add(name.clone(), abs_path.clone(), path.is_dir(), ext.to_string(), Some(content.to_string()));
                //IDX_STORE.add_text(file_name, content);
                //continue;
              //}
            }
            //IDX_STORE.add(name.clone(), abs_path, path.is_dir(), ext.to_string(),None)
          }
        }
        Ok(event) => error!("broken event: {:?}", event),
        Err(e) => error!("watch error: {:?}", e),
      }
    }
  }

  fn save_subs(&mut self, parent_str: &str) {
    let subs = subs(parent_str);
    for sub in subs {
      let sub_path = path::Path::new(sub.as_str());
      let name = sub_path
        .file_name()
        .map(|x| x.to_str().unwrap())
        .unwrap_or_default()
        .to_string();

      if let Ok(meta) = sub_path.metadata() {
        let name0 = name.clone();
        let ext = utils::file_ext(&name0);
        IDX_STORE.add(name, sub.clone(), meta.is_dir(), ext.to_string(),None);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::idx_store::IdxStore;
  use notify::watcher;
  use std::sync::Arc;
  use std::time::Duration;

  #[test]
  fn t1() {

    // let mut watcher = FsWatcher::new(
    //   ,
    //   "/Users/jeff/CLionProjects/orangemac/src-tauri/target".to_string(),
    // );
    // watcher.start();
  }
  #[test]
  fn t2() {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
      .watch(
        "/Users/jeff/CLionProjects/orangemac/src-tauri/target/hi",
        RecursiveMode::Recursive,
      )
      .unwrap();

    loop {
      match rx.recv() {
        Ok(event) => println!("{:?}", event),
        Err(e) => println!("watch error: {:?}", e),
      }
    }
  }
  #[test]
  fn t3() {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering raw events.
    // The notification back-end is selected based on the platform.
    let mut watcher = raw_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch("/", RecursiveMode::Recursive).unwrap();

    loop {
      match rx.recv() {
        Ok(RawEvent {
          path: Some(path),
          op: Ok(_op),
          cookie: _,
        }) => {
          let x = path.to_str().unwrap();
          if x.contains("orangecachedata") {
            continue;
          }
          println!("{}", x);
          // println!("{:?} {:?} ({:?})", op, path, cookie)
        }
        Ok(event) => println!("broken event: {:?}", event),
        Err(e) => println!("watch error: {:?}", e),
      }
    }
  }

  #[test]
  fn t4() {
    let _conf_path = format!("{}{}", utils::data_dir(), "/orangecachedata/conf");
    let idx_path = format!("{}{}", utils::data_dir(), "/orangecachedata/idx");

    let _idx_store = Arc::new(IdxStore::new(&idx_path));
    let mut watcher = FsWatcher::new("/".to_string());
    watcher.start();
  }
}
