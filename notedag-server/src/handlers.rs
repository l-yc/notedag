use std::ffi::OsString;
use std::fs;

use std::convert::Infallible;
use std::env;
use std::io::Write;
use std::time::SystemTime;

use futures_util::TryFutureExt;

use crate::kernel::KernelSpec;
use crate::models::ListItem;
use crate::models::ListOptions;
use crate::models::NoteDAG;
use crate::models::NoteDAGWrite;

fn get_path(file_path: &str) -> String {
    let root = env::var_os("ROOT")
        .map(OsString::into_string)
        .transpose()
        .unwrap()
        .unwrap_or_default();
    format!("{}{}", root, file_path)
}

pub async fn list(options: ListOptions) -> Result<impl warp::Reply, Infallible> {
    let dir = options.file_path.unwrap_or_default();
    let paths = fs::read_dir(get_path(&dir)).unwrap();

    println!("Listing {}", dir);
    let mut files = vec![];
    for path in paths {
        println!("Name: {:?}", path);

        let f = path.unwrap();
        let metadata = f.metadata().unwrap();

        let file_name = f.file_name().into_string().unwrap();
        let file_path = f.path().into_os_string().into_string().unwrap();
        let is_dir = f.file_type().unwrap().is_dir();
        let size = metadata.len();
        let modified = metadata.modified().unwrap().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
        files.push(ListItem {
            file_name,
            file_path,
            is_dir,
            size,
            modified,
        });
    }

    Ok(warp::reply::json(&files))
}

pub async fn create(notedag: NoteDAG) -> Result<impl warp::Reply, Infallible> {
    let path = get_path(&notedag.file_path);
    let file = fs::File::create(path).unwrap();
    println!("Created {:?}", file);
    Ok(warp::reply())
}

pub async fn read(notedag: NoteDAG) -> Result<impl warp::Reply, Infallible> {
    let path = get_path(&notedag.file_path);
    let contents = fs::read_to_string(&path).unwrap();
    println!("Read {}", path);
    Ok(warp::reply::json(&contents))
}

pub async fn write(notedag: NoteDAGWrite) -> Result<impl warp::Reply, Infallible> {
    let path = get_path(&notedag.file_path);
    let _ = fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .write_all(&notedag.contents.into_bytes())
        .unwrap();
    println!("Wrote {}", path);
    Ok(warp::reply())
}

pub async fn list_kernels() -> Result<impl warp::Reply, Infallible> {
    let kernels = KernelSpec::get_available_kernels().unwrap_or_else(|_| vec![]);
    Ok(warp::reply::json(&kernels))
}

