use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=todos=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "notedag=info");
    }
    pretty_env_logger::init();

    let api = filters::api();

    let routes = api.with(warp::log("todos"));

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

mod filters {
    use serde::de::DeserializeOwned;
    use warp::Filter;
    use crate::models;
    use crate::handlers;

    pub fn api() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        // GET /hello/warp => 200 OK with body "Hello, warp!"
        //warp::path!("checkhealth")
        //    .map(|| warp::reply())
        //    .or(list())
        //    .or(create())
        checkhealth().or(notedag())
    }

    fn checkhealth() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        // GET /checkhealth => 200 OK
        warp::path!("checkhealth")
            .map(warp::reply)
    }

    fn notedag() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("notedag")
            .and(list()
                 .or(create())
                 .or(read())
                 .or(write()))
    }

    fn list() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("list")
            .and(warp::get())
            .and(warp::query::<models::ListOptions>())
            .and_then(handlers::list)
    }

    fn create() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("create")
            .and(warp::post())
            .and(json_body())
            .and_then(handlers::create)
    }

    fn read() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("read")
            .and(warp::get())
            .and(warp::query::<models::NoteDAG>())
            .and_then(handlers::read)
    }

    fn write() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("write")
            .and(warp::post())
            .and(json_body())
            .and_then(handlers::write)
    }

    fn json_body<T: Send + DeserializeOwned>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct NoteDAG {
        pub file_path: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct NoteDAGWrite {
        pub file_path: String,
        pub contents: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ListItem {
        pub file_name: String,
        pub file_path: String,
        pub is_dir: bool,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ListOptions {
        pub file_path: Option<String>,
    }
}

mod handlers {
    use std::ffi::OsString;
    use std::fs;

    use std::convert::Infallible;
    use std::env;
    use std::io::Write;

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
            let file_name = f.file_name().into_string().unwrap();
            let file_path = f.path().into_os_string().into_string().unwrap();
            let is_dir = f.file_type().unwrap().is_dir();
            files.push(ListItem {
                file_name,
                file_path,
                is_dir,
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
        let _ = fs::OpenOptions::new().write(true)
            .open(&path).unwrap()
            .write_all(&notedag.contents.into_bytes()).unwrap();
        println!("Wrote {}", path);
        Ok(warp::reply())
    }
}
