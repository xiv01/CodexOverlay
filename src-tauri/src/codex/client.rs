use super::{process, protocol};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStdin},
    sync::{mpsc, oneshot, Mutex},
    time::{timeout, Duration},
};

#[derive(Debug)]
pub enum ClientError {
    Process(String),
    Authentication,
    Timeout,
    Server(String),
    Closed,
}

pub struct CodexAppServerClient {
    writer: Arc<Mutex<BufWriter<ChildStdin>>>,
    child: Arc<Mutex<Child>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    next_id: AtomicU64,
    notifications: mpsc::UnboundedReceiver<Value>,
}

impl CodexAppServerClient {
    pub async fn spawn(custom_path: Option<&str>) -> Result<Self, ClientError> {
        let executable = process::discover(custom_path).await.map_err(|e| match e {
            process::ProcessError::NotFound => ClientError::Process("Codex CLI not found".into()),
            process::ProcessError::Start(s) => ClientError::Process(s),
        })?;
        let mut child = process::spawn(executable).await.map_err(|e| match e {
            process::ProcessError::NotFound => ClientError::Process("Codex CLI not found".into()),
            process::ProcessError::Start(s) => ClientError::Process(s),
        })?;
        let stdin = child.stdin.take().ok_or(ClientError::Closed)?;
        let stdout = child.stdout.take().ok_or(ClientError::Closed)?;
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = mpsc::unbounded_channel();
        let reader_pending = pending.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                match serde_json::from_str::<Value>(&line) {
                    Ok(value) => {
                        if let Some(id) = protocol::response_id(&value) {
                            if let Some(waiter) = reader_pending.lock().await.remove(&id) {
                                let _ = waiter.send(value);
                            }
                        } else if protocol::notification_method(&value).is_some() {
                            let _ = sender.send(value);
                        }
                    }
                    Err(_) => tracing::warn!("discarded malformed app-server JSONL message"),
                }
            }
        });
        Ok(Self {
            writer: Arc::new(Mutex::new(BufWriter::new(stdin))),
            child: Arc::new(Mutex::new(child)),
            pending,
            next_id: AtomicU64::new(1),
            notifications: receiver,
        })
    }

    pub async fn initialize(&self) -> Result<(), ClientError> {
        let response = self.send_request_with_id(1, "initialize", json!({"clientInfo":{"name":"codex_usage_overlay","title":"Codex Usage Overlay","version":"0.1.0"}}), Duration::from_secs(10)).await?;
        if response.get("error").is_some() {
            return Err(ClientError::Server(protocol::sanitized_error(&response)));
        }
        self.send_notification("initialized", json!({})).await
    }

    pub async fn send_request(
        &self,
        method: &str,
        params: Value,
        deadline: Duration,
    ) -> Result<Value, ClientError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed).max(2);
        self.send_request_with_id(id, method, params, deadline)
            .await
    }

    async fn send_request_with_id(
        &self,
        id: u64,
        method: &str,
        params: Value,
        deadline: Duration,
    ) -> Result<Value, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        self.write(json!({"method":method,"id":id,"params":params}))
            .await?;
        match timeout(deadline, rx).await {
            Ok(Ok(response)) if response.get("error").is_none() => Ok(response),
            Ok(Ok(response)) if protocol::is_auth_error(&response) => {
                Err(ClientError::Authentication)
            }
            Ok(Ok(response)) => Err(ClientError::Server(protocol::sanitized_error(&response))),
            Ok(Err(_)) => Err(ClientError::Closed),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(ClientError::Timeout)
            }
        }
    }

    pub async fn send_notification(&self, method: &str, params: Value) -> Result<(), ClientError> {
        self.write(json!({"method":method,"params":params})).await
    }
    async fn write(&self, value: Value) -> Result<(), ClientError> {
        let mut writer = self.writer.lock().await;
        writer
            .write_all(value.to_string().as_bytes())
            .await
            .map_err(|_| ClientError::Closed)?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|_| ClientError::Closed)?;
        writer.flush().await.map_err(|_| ClientError::Closed)
    }
    pub async fn read_account(&self) -> Result<Value, ClientError> {
        self.send_request(
            "account/read",
            json!({"refreshToken":false}),
            Duration::from_secs(10),
        )
        .await
    }
    pub async fn refresh_rate_limits(&self) -> Result<Value, ClientError> {
        self.send_request(
            "account/rateLimits/read",
            json!({}),
            Duration::from_secs(10),
        )
        .await
    }
    pub async fn next_notification(&mut self) -> Option<Value> {
        self.notifications.recv().await
    }
    pub async fn shutdown(&self) {
        let _ = self.child.lock().await.kill().await;
    }
    pub async fn restart(custom_path: Option<&str>) -> Result<Self, ClientError> {
        Self::spawn(custom_path).await
    }
}
