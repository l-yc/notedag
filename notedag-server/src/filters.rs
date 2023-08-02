use warp::Filter;

pub fn api(
    notify_shutdown: tokio::sync::broadcast::Receiver<()>,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    checkhealth().or(notedag::main()).or(kernel::main(notify_shutdown, shutdown_complete_tx))
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
    use crate::handlers;

    use crate::kernel::Kernel;
    use crate::kernel::KernelConnection;
    use crate::kernel::KernelSpec;
    use crate::models::KernelUpdate;
    use crate::models::RunCell;
    use crate::models::RunCellUpdate;
    use jupyter_client::responses::ExecutionState;
    use warp::Filter;

    use std::collections::HashMap;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use futures_util::{SinkExt, StreamExt, TryFutureExt};
    use tokio::sync::{mpsc, Mutex, RwLock};
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use warp::ws::{Message, WebSocket};

    use jupyter_client::responses::{IoPubResponse, Response};

    /// Our global unique user id counter.
    static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

    /// Our state of currently connected users.
    ///
    /// - Key is their id
    /// - Value is a sender of `warp::ws::Message`
    type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;

    pub fn main(
        notify_shutdown: tokio::sync::broadcast::Receiver<()>,
        shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("kernel").and(list().or(socket(notify_shutdown, shutdown_complete_tx)))
    }

    fn list() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path!("list")
            .and(warp::get())
            .and_then(handlers::list_kernels)
    }

    // GET /kernel/socket -> websocket upgrade
    pub fn socket(
        _notify_shutdown: tokio::sync::broadcast::Receiver<()>,
        _shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        // Keep track of all connected users, key is usize, value
        // is a websocket sender.
        let users = Users::default();
        // Turn our "state" into a new Filter...
        let users = warp::any().map(move || users.clone());


        warp::path("socket")
            // The `ws()` filter will prepare Websocket handshake...
            .and(warp::ws())
            .and(users)
            .map(|ws: warp::ws::Ws, users| {
                // This will call our function if the handshake succeeds.
                ws.on_upgrade(move |socket| user_connected(socket, users))
            })
    }

    async fn user_connected(
        ws: WebSocket,
        users: Users,
    ) {
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
        let spec: Vec<KernelSpec> = KernelSpec::get_available_kernels().unwrap();
        let kernel = Kernel::start(&spec[0]).await.unwrap();

        let conn = kernel.connect().await.unwrap();
        let new_msg = serde_json::to_string(&KernelUpdate { status: "ready".into() }).unwrap();
        tx.send(Message::text(new_msg)).unwrap();        

        // Set up the heartbeat watcher
        let hb_receiver = conn.client.heartbeat().unwrap();
        std::thread::spawn(move || {
            for _ in hb_receiver {
                debug!("Received heartbeat from kernel");
            }
        });

        // Spawn an IOPub watcher
        {
            let tx = tx.clone();
            let last_run_cell = Arc::clone(&conn.last_run_cell);
            let receiver = conn.client.iopub_subscribe().unwrap();
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
                            tx.send(thing.into()).unwrap();
                        }
                    }
                }
            });
        }

        {
            let conn = Arc::new(Mutex::new(conn));

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
                    tx.clone(),
                    Arc::clone(&conn),
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
        tx: mpsc::UnboundedSender<Message>,
        conn: Arc<Mutex<KernelConnection>>,
    ) {
        // Skip any non-Text messages...
        let msg = if let Ok(s) = msg.to_str() {
            s
        } else {
            return;
        };
        println!("received from {}: {}", my_id, msg);

        let run_cell: RunCell = serde_json::from_str(msg).unwrap();

        // Send update to subscriber
        let _ = tx.send(RunCellUpdate {
            id: run_cell.id.clone(),
            name: String::from("queued"),
            value: ":".into(),
        }.into());

        // This is a slow but blocking step, so we're going to toss it into a tokio spawn.
        tokio::task::spawn(async move {
            let conn = conn.lock().await;
            let execution_count = conn.run_cell(run_cell.clone()).unwrap();

            let _ = tx.send(RunCellUpdate {
                id: run_cell.id.clone(),
                name: String::from("count"),
                value: execution_count.to_string(),
            }.into());
        });
    }

    async fn user_disconnected(my_id: usize, users: &Users) {
        eprintln!("good bye user: {}", my_id);

        // Stream closed up, so remove from the user list
        users.write().await.remove(&my_id);
    }
}
