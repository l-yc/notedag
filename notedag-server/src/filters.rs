use warp::Filter;

pub fn api() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    checkhealth().or(notedag::main()).or(kernel::main())
}

fn checkhealth() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    // GET /checkhealth => 200 OK
    warp::path!("checkhealth").map(warp::reply)
}

mod notedag {
    use crate::handlers;
    use crate::models;
    use serde::de::DeserializeOwned;
    use warp::Filter;


    pub fn main() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("notedag").and(list().or(create()).or(read()).or(write()))
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

    fn json_body<T: Send + DeserializeOwned>(
    ) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body
        // (and to reject huge payloads)...
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

mod kernel {
    use crate::models::KernelUpdate;
    use crate::models::RunCell;
    use crate::models::RunCellUpdate;
    use jupyter_client::responses::ExecutionState;
    use tokio::sync::mpsc::UnboundedSender;
    use warp::Filter;

    use std::collections::HashMap;
    use std::process::Child;
    use std::process;
    use std::sync::{
        self,
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use futures_util::{SinkExt, StreamExt, TryFutureExt};
    use tokio::sync::{mpsc, Mutex, RwLock};
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use warp::ws::{Message, WebSocket};

    use jupyter_client::commands::Command;
    use jupyter_client::Client;
    use jupyter_client::responses::{IoPubResponse, Response, ShellResponse};

    /// Our global unique user id counter.
    static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

    /// Our state of currently connected users.
    ///
    /// - Key is their id
    /// - Value is a sender of `warp::ws::Message`
    type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

    // GET /kernel -> websocket upgrade
    pub fn main() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        // Keep track of all connected users, key is usize, value
        // is a websocket sender.
        let users = Users::default();
        // Turn our "state" into a new Filter...
        let users = warp::any().map(move || users.clone());


        let _kernel: Child = process::Command::new("ipython")
            .args(["kernel", "-f", "./kernel.json"])
            .spawn()
            .unwrap();
        // FIXME obviously not a good solution
        std::thread::sleep(std::time::Duration::from_millis(5000));
        println!("kernel started");


        warp::path("kernel")
            // The `ws()` filter will prepare Websocket handshake...
            .and(warp::ws())
            .and(users)
            .map(|ws: warp::ws::Ws, users| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| user_connected(socket, users))
            })
    }

    async fn user_connected(ws: WebSocket, users: Users) {
        // Use a counter to assign a new unique ID for this user.
        let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

        println!("new user: {}", my_id);

        // Split the socket into a sender and receive of messages.
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();

        // Use an unbounded channel to handle buffering and flushing of messages
        // to the websocket...
        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        {
            let tx = tx.clone();
            tokio::task::spawn(async move {
                while let Some(message) = rx.next().await {
                    user_ws_tx
                        .send(message)
                        .unwrap_or_else(|e| {
                            eprintln!("websocket send error: {}", e);
                        })
                        .await;
                }
            });

            // Save the sender in our list of connected users.
            users.write().await.insert(my_id, tx);
        }

        // Return a `Future` that is basically a state machine managing
        // this specific user's connection.

        let file = std::fs::File::open("./kernel.json").unwrap();
        //let client = Client::existing().unwrap(); // doesn't work
        let client = Client::from_reader(file).unwrap();
        println!("connected to kernel");
        let new_msg = serde_json::to_string(&KernelUpdate { status: "ready".into() }).unwrap();
        tx.send(Message::text(new_msg)).unwrap();


        // Set up the heartbeat watcher
        let hb_receiver = client.heartbeat().unwrap();
        std::thread::spawn(move || {
            for _ in hb_receiver {
                //println!("Received heartbeat from kernel");
            }
        });

        let last_run_cell: Arc<sync::RwLock<Option<RunCell>>> = Arc::new(sync::RwLock::new(None));

        // Spawn an IOPub watcher
        {
            let tx = tx.clone();
            let last_run_cell = Arc::clone(&last_run_cell);
            let receiver = client.iopub_subscribe().unwrap();
            std::thread::spawn(move || {
                for msg in receiver {
                    //println!("Received message from kernel: {:#?}", msg);
                    if let Response::IoPub(response) = msg {
                        let opt = last_run_cell.read().unwrap();
                        let output = opt.as_ref().and_then(|run_cell| {
                            //dbg!(&response);
                            dbg!("received IoPub response");
                            match response {
                                IoPubResponse::Stream { content, .. } => Some(RunCellUpdate {
                                    id: run_cell.id.clone(),
                                    name: String::from("output"),
                                    value: content.text,
                                }),
                                IoPubResponse::Error { content, .. } => {
                                    Some(RunCellUpdate {
                                        id: run_cell.id.clone(),
                                        name: String::from("error"),
                                        value: content.traceback.join("\n"),
                                    })
                                },
                                IoPubResponse::Status { content, .. } => {
                                    println!("status update: {:?}", content);
                                    match content.execution_state {
                                        ExecutionState::Busy => None,
                                        ExecutionState::Idle => Some(RunCellUpdate {
                                            id: run_cell.id.clone(),
                                            name: String::from("complete"),
                                            value: String::from(""),
                                        }),
                                        _ => None,
                                    }
                                },
                                IoPubResponse::ExecuteInput { .. } => {
                                    Some(RunCellUpdate {
                                        id: run_cell.id.clone(),
                                        name: String::from("running"),
                                        value: "*".into(),
                                    })
                                },
                                IoPubResponse::ExecuteResult { content, .. } => {
                                    Some(RunCellUpdate {
                                        id: run_cell.id.clone(),
                                        name: String::from("result"),
                                        value: serde_json::to_string(&content.data).unwrap()
                                    })
                                },
                                IoPubResponse::DisplayData { content, .. } => {
                                    Some(RunCellUpdate {
                                        id: run_cell.id.clone(),
                                        name: String::from("data"),
                                        value: serde_json::to_string(&content.data).unwrap()
                                    })
                                },
                                _ => None
                            }
                        });

                        if let Some(thing) = output {
                            let new_msg = serde_json::to_string(&thing).unwrap();
                            tx.send(Message::text(new_msg)).unwrap();
                        }
                    }
                }
            });
        }

        {
            let client = Arc::new(Mutex::new(client));

            while let Some(result) = user_ws_rx.next().await {
                let msg = match result {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("websocket error(uid={}): {}", my_id, e);
                        break;
                    }
                };
                user_message(
                    my_id,
                    msg,
                    &tx,
                    Arc::clone(&client),
                    Arc::clone(&last_run_cell),
                )
                .await;
            }
        }

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        user_disconnected(my_id, &users).await;
    }

    async fn user_message(
        my_id: usize,
        msg: Message,
        tx: &UnboundedSender<Message>,
        client: Arc<Mutex<Client>>,
        last_run_cell: Arc<sync::RwLock<Option<RunCell>>>,
    ) {
        // Skip any non-Text messages...
        let msg = if let Ok(s) = msg.to_str() {
            s
        } else {
            return;
        };
        println!("received from {}: {}", my_id, msg);

        let run_cell: RunCell = serde_json::from_str(msg).unwrap();
        {
            let mut r = last_run_cell.write().unwrap();
            *r = Some(run_cell.clone());
        }
        println!("submitting: {}", run_cell.value);

        // Command to run
        let command = Command::Execute {
            code: run_cell.value.to_string(),
            silent: false,
            store_history: true,
            user_expressions: HashMap::new(),
            allow_stdin: true,
            stop_on_error: false,
        };

        let cell_output = RunCellUpdate {
            id: run_cell.id.clone(),
            name: String::from("queued"),
            value: ":".into(),
        };

        let new_msg = serde_json::to_string(&cell_output).unwrap();
        tx.send(Message::text(new_msg)).unwrap();

        // Run some code on the kernel
        // This is a slow but blocking step, so we're going to toss it into a tokio spawn.
        let response = tokio::task::spawn(async move {
            let c = client.lock().await;
            c.send_shell_command(command).unwrap()
        })
        .await;

        if let Ok(Response::Shell(ShellResponse::Execute { content, .. })) = response {
            dbg!(content.execution_count);
            let cell_output = RunCellUpdate {
                id: run_cell.id.clone(),
                name: String::from("count"),
                value: content.execution_count.to_string(),
            };

            let new_msg = serde_json::to_string(&cell_output).unwrap();

            tx.send(Message::text(new_msg)).unwrap();
        }
    }

    async fn user_disconnected(my_id: usize, users: &Users) {
        eprintln!("good bye user: {}", my_id);

        // Stream closed up, so remove from the user list
        users.write().await.remove(&my_id);
    }
}
