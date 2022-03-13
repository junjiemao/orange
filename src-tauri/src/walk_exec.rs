use crate::idx_store::IdxStore;
use crate::kv_store::KvStore;

use crate::utils;
#[cfg(windows)]
use crate::utils::get_win32_ready_drives;

use jwalk::WalkDir;
use std::sync::Arc;
use std::time::SystemTime;

pub fn home_dir() -> String {
  let option = dirs::home_dir();
  option.unwrap().to_str().unwrap().to_string()
}

pub fn run(conf_store: Arc<KvStore>, idx_store: Arc<IdxStore>) {
  let home = utils::norm(&home_dir());

  walk_home(conf_store.clone(), idx_store.clone(), &home);

  #[cfg(windows)]
  win_walk_root(conf_store, idx_store, home);

  #[cfg(unix)]
  unix_walk_root(conf_store, idx_store, home);
}
fn unix_walk_root(conf_store: Arc<KvStore>, idx_store: Arc<IdxStore>, home: String) {
  let key = format!("walk:stat:{}", "/");
  let opt = conf_store.get_str(key.clone());
  if opt.is_some() {
    return;
  }
  walk(idx_store.clone(), &"/".to_string(), Some(home.to_string()));
  conf_store.put_str(key, "1".to_string());
}

#[cfg(windows)]
fn win_walk_root(conf_store: Arc<KvStore>, idx_store: Arc<IdxStore>, home: String) {
  let drives = unsafe { get_win32_ready_drives() };

  for mut driv in drives {
    driv = utils::norm(&driv);
    let key = format!("walk:stat:{}", &driv);
    let opt = conf_store.get_str(key.clone());
    if opt.is_some() {
      return;
    }

    walk(idx_store.clone(), &driv, Some(home.to_string()));
    conf_store.put_str(key, "1".to_string());
  }
}

fn walk_home(conf_store: Arc<KvStore>, idx_store: Arc<IdxStore>, home: &String) {
  let key = format!("walk:stat:{}", home);
  let opt = conf_store.get_str(key.clone());
  if opt.is_some() {
    return;
  }

  let home_name = utils::path2name(&home).unwrap_or("");
  idx_store.add(home_name, &home);
  walk(idx_store, &home, None);
  conf_store.put_str(key, "1".to_string());
}

fn walk(store: Arc<IdxStore>, path: &String, skip_path_opt: Option<String>) {
  let start = SystemTime::now();
  println!("start travel {}", path);
  let mut cnt = 0;

  let mut generic = WalkDir::new(&path);
  if skip_path_opt.is_some() {
    let skip_path = skip_path_opt.unwrap();
    generic = generic.process_read_dir(move |_depth, _path, _read_dir_state, children| {
      children.iter_mut().for_each(|dir_entry_result| {
        if let Ok(dir_entry) = dir_entry_result {
          if utils::norm(dir_entry.path().to_str().unwrap_or("")).eq(skip_path.as_str()) {
            dir_entry.read_children_path = None;
          }
        }
      });
    });
  }

  for entry in generic {
    cnt += 1;
    let en = entry.unwrap();
    let buf = en.path();
    let path = buf.to_str().unwrap();
    let name = en.file_name().to_str().unwrap();

    store.add(name, path);
  }
  let end = SystemTime::now();
  store.commit();
  println!(
    "cost {} s, total {} files",
    end.duration_since(start).unwrap().as_secs(),
    cnt
  );
}

#[test]
fn t1() {
  let conf_path = format!("{}{}", utils::data_dir(), "/orangecachedata/conf");
  let idx_path = format!("{}{}", utils::data_dir(), "/orangecachedata/idx");

  let conf_store = Arc::new(KvStore::new(&conf_path));
  let idx_store = Arc::new(IdxStore::new(&idx_path));

  run(conf_store, idx_store);
}
