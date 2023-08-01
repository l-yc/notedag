use std::{process, collections::HashMap, fs};
use jupyter_client::{Client, commands::Command, responses::{Response, ShellResponse}};
use serde::Serialize;
use uuid::Uuid;

use std::sync::{
    self,
    Arc,
};

use crate::models::RunCell;

#[derive(Clone, Debug, Serialize)]
pub struct KernelSpec {
    pub cmd: String,
    pub args: Vec<String>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl KernelSpec {
    pub fn get_available_kernels() -> Result<Vec<Self>> {
        Ok(vec![
           KernelSpec {
               cmd: "ipython".into(),
               args: ["kernel".into(), "-f".into()].into(),
           }
        ])
    }
}

pub struct KernelConnection {
    pub client: Client,
    pub last_run_cell: Arc<sync::RwLock<Option<RunCell>>>,
}

impl KernelConnection {
    pub fn run_cell(&self, run_cell: RunCell) -> Result<i64> {
        // update last_run_cell
        let mut r = self.last_run_cell.write().unwrap();
        *r = Some(run_cell.clone());

        // submit code to kernel
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

        // Run some code on the kernel - slow step
        let response = self.client.send_shell_command(command);

        if let Ok(Response::Shell(ShellResponse::Execute { content, .. })) = response {
            dbg!(content.execution_count);
            Ok(content.execution_count)
        } else {
            Err("failed to run code".into())
        }
    }
}

pub struct Kernel {
    pub spec: KernelSpec,
    process: process::Child,
    pub file: String,
}

impl Drop for Kernel {
    fn drop(&mut self) {
        // kill child once the Kernel object goes out of scope
        self.shutdown().expect("Failed to shutdown kernel");
    }
}

impl Kernel {
    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down kernel...");
        self.process.kill()?; // FIXME obviously not a good way either
        fs::remove_file(&self.file)?;
        Ok(())
    }

    pub async fn start(spec: &KernelSpec) -> Result<Self> {
        let file = format!("./kernel-{}.json", Uuid::new_v4());
        let mut args = spec.args.to_vec();
        args.push(file.clone());
        let args = args;

        let process = process::Command::new(&spec.cmd)
            .args(&args)
            .spawn()?;

        let kernel = Kernel {
            spec: spec.clone(),
            process,
            file,
        };

        // FIXME obviously not a good solution
        std::thread::sleep(std::time::Duration::from_millis(5000));
        info!("kernel started");
        Ok(kernel)
    }

    pub async fn connect(&self) -> Result<KernelConnection> {
        let file = std::fs::File::open(&self.file).unwrap();
        //let client = Client::existing().unwrap(); // doesn't work
        let client = Client::from_reader(file).unwrap();
        info!("connected to kernel");

        Ok(KernelConnection { 
            client,
            last_run_cell: Arc::new(sync::RwLock::new(None)),
        })
    }
}
