#![allow(unused)]
use async_trait::async_trait;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::async_node::AsyncNode;
use crate::errors::{FlowError, FlowResult};
use crate::sync::{BaseNode, Node, NodeRef, NodeResult, Params, Shared, SyncNode};

// Helper function to create server connection errors
fn server_conn_error(msg: &str) -> FlowError {
    FlowError::MCPServer(msg.to_string())
}

/// Configuration for an MCP client node
#[derive(Clone)]
pub struct MCPClientConfig {
    /// Optional server name to connect to
    pub server_name: Option<String>,
    /// Optional server command to launch
    pub server_command: Option<String>,
    /// Optional server arguments
    pub server_args: Vec<String>,
    /// Optional client name to identify with
    pub client_name: String,
    /// Optional client version
    pub client_version: String,
}

impl Default for MCPClientConfig {
    fn default() -> Self {
        MCPClientConfig {
            server_name: None,
            server_command: None,
            server_args: Vec::new(),
            client_name: "RPocketFlow MCP Client".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl MCPClientConfig {
    pub fn new(client_name: impl Into<String>, client_version: impl Into<String>) -> Self {
        MCPClientConfig {
            server_name: None,
            server_command: None,
            server_args: Vec::new(),
            client_name: client_name.into(),
            client_version: client_version.into(),
        }
    }

    pub fn with_server_name(mut self, server_name: impl Into<String>) -> Self {
        self.server_name = Some(server_name.into());
        self
    }

    pub fn with_server_command(mut self, command: impl Into<String>, args: Vec<String>) -> Self {
        self.server_command = Some(command.into());
        self.server_args = args;
        self
    }
}

/// Type representing an MCP client result
pub type MCPClientResult<T> = crate::errors::FlowResult<T>;

/// A struct to handle communication with the MCP server process
struct MCPConnection {
    stdin_tx: mpsc::Sender<String>,
    stdout_rx: mpsc::Receiver<String>,
}

// The struct is not Clone, but we'll provide methods to share it safely
impl MCPConnection {
    /// Create a new MCPConnection with the given server command and arguments
    async fn new(command: &str, args: &[String]) -> crate::errors::FlowResult<(Self, Child)> {
        use crate::errors::FlowError;
        
        // Start the server process
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| FlowError::MCPServer(format!("Failed to start MCP server process: {}", e)))?;

        // Get stdin and stdout
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| FlowError::MCPServer("Failed to open stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| FlowError::MCPServer("Failed to open stdout".to_string()))?;

        // Create channels for communication
        let (stdin_tx, mut stdin_rx) = mpsc::channel(10);
        let (stdout_tx, stdout_rx) = mpsc::channel(10);

        // Spawn a task to handle stdin
        let _stdin_handle = tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(message) = stdin_rx.recv().await {
                if let Err(e) = writeln!(stdin, "{}", message) {
                    eprintln!("Failed to write to stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush() {
                    eprintln!("Failed to flush stdin: {}", e);
                    break;
                }
            }
        });

        // Spawn a task to handle stdout
        let _stdout_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Some(Ok(line)) = lines.next() {
                if let Err(e) = stdout_tx.send(line).await {
                    eprintln!("Failed to send stdout line: {}", e);
                    break;
                }
            }
        });

        let connection = MCPConnection {
            stdin_tx,
            stdout_rx,
        };

        Ok((connection, child))
    }

    /// Clone the sender to allow multiple references to communicate with the server
    fn clone_sender(&self) -> mpsc::Sender<String> {
        self.stdin_tx.clone()
    }

    /// Send a message to the server
    async fn send(&self, message: &Value) -> Result<(), String> {
        let json_str = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        self.stdin_tx
            .send(json_str)
            .await
            .map_err(|e| format!("Failed to send message: {}", e))
    }

    /// Receive a message from the server
    async fn receive<T: for<'de> Deserialize<'de>>(&mut self) -> Result<T, String> {
        let json_str = self
            .stdout_rx
            .recv()
            .await
            .ok_or_else(|| "Connection closed".to_string())?;

        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to deserialize response: {}", e))
    }
}

/// A node that implements the MCP client protocol
pub struct MCPProtocolNode {
    base: BaseNode,
    config: MCPClientConfig,
    connection: Option<MCPConnection>,
    child_process: Option<Child>,
    available_tools: HashMap<String, String>, // tool_name -> description
    last_result: Option<Value>,
    next_id: u64,
}

impl MCPProtocolNode {
    pub fn new(name: impl Into<String>, config: MCPClientConfig) -> Self {
        MCPProtocolNode {
            base: BaseNode::new(name),
            config,
            connection: None,
            child_process: None,
            available_tools: HashMap::new(),
            last_result: None,
            next_id: 1,
        }
    }

    /// Initialize the MCP client
    pub async fn initialize(&mut self) -> MCPClientResult<Value> {
        // Check if we have a server command to launch
        if let Some(command) = &self.config.server_command {
            // Launch the server process and create a connection
            let (connection, child) = MCPConnection::new(command, &self.config.server_args).await?;

            self.connection = Some(connection);
            self.child_process = Some(child);

            info!("Started MCP server process");
        } else {
            return Err(crate::errors::FlowError::MCPServer("No server command specified".to_string()));
        }

        // Prepare initialization message
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "initialize",
            "params": {
                "clientName": self.config.client_name,
                "clientVersion": self.config.client_version,
                "capabilities": {}
            }
        });
        self.next_id += 1;

        // Send initialization request
        if let Some(conn) = &self.connection {
            conn.send(&init_request).await?;
        } else {
            return Err(crate::errors::FlowError::MCPServer("No connection to MCP server".to_string()));
        }

        // Receive the response
        let response: Value = if let Some(conn) = &mut self.connection {
            conn.receive().await?
        } else {
            return Err(server_conn_error("No connection to MCP server"));
        };

        // Extract and parse the result
        if let Some(result) = response.get("result") {
            // Parse server info from the response
            if let Some(server_info) = result.get("serverInfo") {
                if let (Some(name), Some(version)) = (
                    server_info.get("name").and_then(|v| v.as_str()),
                    server_info.get("version").and_then(|v| v.as_str()),
                ) {
                    let protocol_version = result
                        .get("protocolVersion")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    info!(
                        "Connected to MCP server: {} v{} (protocol {})",
                        name, version, protocol_version
                    );
                }
            }

            Ok(result.clone())
        } else if let Some(error) = response.get("error") {
            Err(FlowError::Protocol(format!("Initialization error: {}", error)))
        } else {
            Err(FlowError::Protocol("Invalid response format".to_string()))
        }
    }

    /// Shutdown the MCP client
    pub async fn shutdown(&mut self) -> MCPClientResult<()> {
        // Prepare shutdown message
        let shutdown_request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": "shutdown",
            "params": {}
        });
        self.next_id += 1;

        // Send shutdown request
        if let Some(conn) = &self.connection {
            // Don't propagate errors here, we want to clean up regardless
            let _ = conn.send(&shutdown_request).await;
        }

        // Kill the child process if it exists
        if let Some(mut child) = self.child_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        // Clear the connection
        self.connection = None;

        Ok(())
    }

    /// Call a tool on the MCP server
    pub async fn call_tool<P: Serialize, R: for<'de> Deserialize<'de>>(
        &mut self,
        tool_name: &str,
        params: &P,
    ) -> MCPClientResult<R> {
        // Prepare tool call request
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": tool_name,
            "params": params
        });
        self.next_id += 1;

        // Send the request
        if let Some(conn) = &self.connection {
            conn.send(&request).await?;
        } else {
            return Err(crate::errors::FlowError::MCPServer("No connection to MCP server".to_string()));
        }

        // Receive the response
        let response: Value = if let Some(conn) = &mut self.connection {
            conn.receive().await?
        } else {
            return Err(server_conn_error("No connection to MCP server"));
        };

        // Extract and parse the result
        if let Some(result) = response.get("result") {
            serde_json::from_value(result.clone())
                .map_err(|e| FlowError::Protocol(format!("Failed to parse result: {}", e)))
        } else if let Some(error) = response.get("error") {
            Err(FlowError::Protocol(format!("Tool call error: {}", error)))
        } else {
            Err(FlowError::Protocol("Invalid response format".to_string()))
        }
    }

    /// Get available tools from the server
    pub async fn get_available_tools(&mut self) -> MCPClientResult<HashMap<String, String>> {
        let tools_result = self.call_tool::<_, Value>("tools/list", &json!({})).await?;

        let mut tool_map = HashMap::new();

        if let Some(tools) = tools_result.get("tools").and_then(|t| t.as_array()) {
            for tool in tools {
                if let (Some(name), Some(description)) = (
                    tool.get("name").and_then(|n| n.as_str()),
                    tool.get("description").and_then(|d| d.as_str()),
                ) {
                    tool_map.insert(name.to_string(), description.to_string());
                }
            }
        }

        // Store the tools in our instance
        self.available_tools = tool_map.clone();

        Ok(tool_map)
    }
}

impl Drop for MCPProtocolNode {
    fn drop(&mut self) {
        // Make sure we terminate the child process when the node is dropped
        if let Some(mut child) = self.child_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

// MCPProtocolNode is already Send and Sync:
// - BaseNode is Send + Sync
// - MCPClientConfig is Clone and all fields are Send + Sync
// - Option<T> is Send + Sync if T is Send + Sync
// - Child is explicitly Send + Sync (documented in std)
// - HashMap<String, String> is Send + Sync
// - u64 is Send + Sync
// The manual impls are not needed

impl Node for MCPProtocolNode {
    fn get_params(&self) -> &Params {
        &self.base.params
    }

    fn set_params(&mut self, params: Params) {
        self.base.params = params;
    }

    fn add_successor(&mut self, action: String, successor: NodeRef) {
        self.base.add_successor(action, successor);
    }

    fn get_successors(&self) -> &HashMap<String, NodeRef> {
        &self.base.successors
    }

    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> {
        &mut self.base.successors
    }

    fn get_name(&self) -> &str {
        &self.base.name
    }
}

impl SyncNode for MCPProtocolNode {}

#[async_trait]
impl AsyncNode for MCPProtocolNode {
    async fn exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        // If tool_name is specified in prep_res, call that tool
        let tool_name = prep_res
            .get("tool_name")
            .and_then(|t| t.as_str())
            .ok_or_else(|| "No tool_name specified in prep_res".to_string())?;

        let params = prep_res.get("params").cloned().unwrap_or_else(|| json!({}));

        debug!("Calling MCP tool: {} with params: {}", tool_name, params);

        // Call the tool and get the result
        match self.call_tool::<_, Value>(tool_name, &params).await {
            Ok(result) => {
                self.last_result = Some(result.clone());
                Ok(json!({
                    "tool_name": tool_name,
                    "result": result,
                    "status": "success"
                }))
            }
            Err(e) => {
                error!("Error calling MCP tool {}: {}", tool_name, e);
                Ok(json!({
                    "tool_name": tool_name,
                    "error": e,
                    "status": "error"
                }))
            }
        }
    }

    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Check if the client needs to be initialized
        if self.connection.is_none() {
            match self.initialize().await {
                Ok(init_result) => {
                    // Get available tools
                    if let Ok(tools) = self.get_available_tools().await {
                        info!("Available MCP tools:");
                        for (name, description) in &tools {
                            info!("  - {} - {}", name, description);
                        }

                        // Store tools in shared state
                        shared.insert("mcp_tools".to_string(), json!(tools));
                    }

                    // Store initialization result in shared state
                    shared.insert("mcp_init_result".to_string(), init_result);
                }
                Err(e) => {
                    return Err(crate::errors::FlowError::MCPClient(format!("Failed to initialize MCP client: {}", e)));
                }
            }
        }

        // Check if we have a tool to call
        let mut tool_name = None;
        let mut params = json!({});

        if let Some(input) = shared.get("mcp_tool_call") {
            tool_name = input
                .get("tool_name")
                .and_then(|t| t.as_str())
                .map(String::from);
            params = input.get("params").cloned().unwrap_or_else(|| json!({}));
        }

        if tool_name.is_none() {
            if let Some(default_tool) = shared.get("mcp_default_tool").and_then(|t| t.as_str()) {
                tool_name = Some(default_tool.to_string());
            }
        }

        // Return the tool to call and its parameters
        Ok(json!({
            "tool_name": tool_name.unwrap_or_else(|| "tools/list".to_string()),
            "params": params,
        }))
    }

    async fn post_async(
        &mut self,
        shared: &mut Shared,
        _prep_res: &Value,
        exec_res: &Value,
    ) -> NodeResult<Value> {
        // Store the most recent result in shared state
        shared.insert("mcp_exec_result".to_string(), exec_res.clone());

        // Determine the next action based on the result
        let status = exec_res
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        if status == "success" {
            Ok(json!("success"))
        } else {
            Ok(json!("error"))
        }
    }
}

/// Create a new MCP Protocol node
pub fn mcp_protocol_node(name: impl Into<String>, config: MCPClientConfig) -> NodeRef {
    let node = MCPProtocolNode::new(name, config);
    Arc::new(Mutex::new(node))
}

