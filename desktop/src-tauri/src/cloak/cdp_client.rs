use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub type CdpResult<T> = Result<T, CdpError>;

#[derive(Debug, thiserror::Error)]
pub enum CdpError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Command {id} failed: {msg}")]
    Command { id: u64, msg: String },
    #[error("Response timeout for command {0}")]
    Timeout(u64),
    #[error("Disconnected")]
    Disconnected,
    #[error("Page not found")]
    PageNotFound,
}

/// 原始 CDP 消息 (JSON-RPC)
#[derive(Debug, Clone)]
pub struct CdpMessage {
    pub id: Option<u64>,
    pub method: Option<String>,
    pub params: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

/// CDP 事件
#[derive(Debug, Clone)]
pub struct CdpEvent {
    pub method: String,
    pub params: Value,
}

/// CDP 客户端
pub struct CdpClient {
    write: Arc<Mutex<futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<CdpMessage>>>>,
    event_tx: broadcast::Sender<CdpEvent>,
    next_id: AtomicU64,
}

impl CdpClient {
    /// 连接到 CloakBrowser 的 CDP WebSocket
    pub async fn connect(ws_url: &str) -> CdpResult<Self> {
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| CdpError::Connection(e.to_string()))?;

        let (write, read) = ws_stream.split();
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<CdpMessage>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (event_tx, _) = broadcast::channel(256);

        let pending_clone = pending.clone();
        let event_tx_clone = event_tx.clone();

        // 后台读任务：接收 CDP 消息并路由
        tokio::spawn(async move {
            let mut read = read;
            let pending = pending_clone;
            let event_tx = event_tx_clone;

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&text) {
                            let msg_id = value.get("id").and_then(|v| v.as_u64());
                            let method = value.get("method").and_then(|v| v.as_str()).map(String::from);

                            if let Some(id) = msg_id {
                                // 这是某个命令的响应
                                let mut pending = pending.lock().await;
                                if let Some(sender) = pending.remove(&id) {
                                    let resp = CdpMessage {
                                        id: Some(id),
                                        method: None,
                                        params: None,
                                        result: value.get("result").cloned(),
                                        error: value.get("error").cloned(),
                                    };
                                    let _ = sender.send(resp);
                                }
                            } else if let Some(method) = method {
                                // 这是事件
                                let event = CdpEvent {
                                    method,
                                    params: value.get("params").cloned().unwrap_or(Value::Null),
                                };
                                let _ = event_tx.send(event);
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }
            log::info!("CDP reader task ended");
        });

        Ok(Self {
            write: Arc::new(Mutex::new(write)),
            pending,
            event_tx,
            next_id: AtomicU64::new(1),
        })
    }

    /// 发送 CDP 命令并等待响应
    pub async fn call_method(&self, method: &str, params: Value) -> CdpResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let cmd = json!({ "id": id, "method": method, "params": params });

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        let msg = Message::Text(cmd.to_string());
        self.write.lock().await.send(msg)
            .await
            .map_err(|_| CdpError::Disconnected)?;

        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(resp)) => {
                if let Some(err) = resp.error {
                    Err(CdpError::Command { id, msg: err.to_string() })
                } else {
                    Ok(resp.result.unwrap_or(Value::Null))
                }
            }
            Ok(Err(_)) => Err(CdpError::Timeout(id)),
            Err(_) => Err(CdpError::Timeout(id)),
        }
    }

    // ── 常用 CDP 命令 ──

    /// 导航到 URL
    pub async fn navigate(&self, url: &str) -> CdpResult<()> {
        self.call_method("Page.navigate", json!({ "url": url })).await?;
        Ok(())
    }

    /// 执行 JavaScript (返回结果字符串)
    pub async fn evaluate(&self, expression: &str) -> CdpResult<String> {
        let result = self.call_method("Runtime.evaluate", json!({
            "expression": expression,
            "returnByValue": true,
        })).await?;
        Ok(result["result"]["value"].as_str().unwrap_or("").to_string())
    }

    /// 启用网络监控
    pub async fn enable_network(&self) -> CdpResult<()> {
        self.call_method("Network.enable", json!({})).await?;
        Ok(())
    }

    /// 启用 Fetch (资源拦截)
    pub async fn enable_fetch(&self) -> CdpResult<()> {
        self.call_method("Fetch.enable", json!({
            "patterns": [{
                "resourceType": "Image",
                "requestStage": "Response"
            }]
        })).await?;
        Ok(())
    }

    /// 设置 Cookie
    pub async fn set_cookies(&self, cookies: &[CdpCookie]) -> CdpResult<()> {
        let cookies_json: Vec<Value> = cookies.iter().map(|c| json!({
            "name": c.name,
            "value": c.value,
            "domain": c.domain,
            "path": c.path,
            "secure": c.secure,
            "httpOnly": c.http_only,
        })).collect();
        self.call_method("Network.setCookies", json!({ "cookies": cookies_json })).await?;
        Ok(())
    }

    /// 获取所有 Cookie
    pub async fn get_cookies(&self) -> CdpResult<Vec<CdpCookie>> {
        let result = self.call_method("Network.getAllCookies", json!({})).await?;
        let cookies: Vec<CdpCookie> = serde_json::from_value(result["cookies"].clone())
            .unwrap_or_default();
        Ok(cookies)
    }

    /// 截屏
    pub async fn capture_screenshot(&self) -> CdpResult<Vec<u8>> {
        let result = self.call_method("Page.captureScreenshot", json!({ "format": "png" })).await?;
        let data = result["data"].as_str().unwrap_or("");
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(data)
            .map_err(|e| CdpError::Command { id: 0, msg: e.to_string() })
    }

    /// 关闭连接
    pub async fn close(&self) {
        let _ = self.write.lock().await.close().await;
    }

    /// 获取事件接收器
    pub fn subscribe(&self) -> broadcast::Receiver<CdpEvent> {
        self.event_tx.subscribe()
    }
}

/// CDP Cookie 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    #[serde(default)]
    pub http_only: bool,
}
