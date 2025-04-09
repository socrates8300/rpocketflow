The **MCPR** crate (Model Context Protocol for Rust) provides a Rust implementation of Anthropic's Model Context Protocol (MCP), an open standard for connecting AI assistants to data sources and tools. Below is a summary of the key details from the documentation:

---

### Overview
The crate includes:
- **Schema Definitions**: For MCP messages.
- **Transport Layer**: For communication.
- **High-Level Client and Server**: Simplified implementations for building MCP applications.
- **CLI Tools**: For generating server and client stubs.
- **Stub Generator**: For creating MCP server and client stubs.

---

### High-Level Client Example
The client provides a simple interface for communicating with MCP servers.

```rust
use mcpr::{
    client::Client,
    transport::stdio::StdioTransport,
};

// Create a client with stdio transport
let transport = StdioTransport::new();
let mut client = Client::new(transport);

// Initialize the client
client.initialize()?;

// Call a tool (example with serde_json::Value)
let request = serde_json::json!({
    "param1": "value1",
    "param2": "value2"
});
let response: serde_json::Value = client.call_tool("my_tool", &request)?;

// Shutdown the client
client.shutdown()?;
```

---

### High-Level Server Example
The server simplifies the creation of MCP-compatible servers.

```rust
use mcpr::{
    error::MCPError,
    server::{Server, ServerConfig},
    transport::stdio::StdioTransport,
    Tool,
};
use serde_json::Value;

// Configure the server
let server_config = ServerConfig::new()
    .with_name("My MCP Server")
    .with_version("1.0.0")
    .with_tool(Tool {
        name: "my_tool".to_string(),
        description: Some("My awesome tool".to_string()),
        input_schema: mcpr::schema::common::ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some([
                ("param1".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "First parameter"
                })),
                ("param2".to_string(), serde_json::json!({
                    "type": "string",
                    "description": "Second parameter"
                }))
            ].into_iter().collect()),
            required: Some(vec!["param1".to_string(), "param2".to_string()]),
        },
    });

// Create the server
let mut server: Server<StdioTransport> = Server::new(server_config);

// Register tool handlers
server.register_tool_handler("my_tool", |params: Value| {
    let param1 = params.get("param1")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError::Protocol("Missing param1".to_string()))?;

    let param2 = params.get("param2")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MCPError::Protocol("Missing param2".to_string()))?;

    let response = serde_json::json!({
        "result": format!("Processed {} and {}", param1, param2)
    });

    Ok(response)
})?;

// In a real application, you would start the server with:
// let transport = StdioTransport::new();
// server.start(transport)?;
```

---

### Key Modules
- **`client`**: High-level client implementation for MCP.
- **`server`**: High-level server implementation for MCP.
- **`transport`**: Transport layer for MCP communication.
- **`schema`**: MCP schema definitions.
- **`generator`**: Module for generating MCP server and client stubs.
- **`cli`**: CLI tools for working with MCP.
- **`error`**: Error types for the MCP implementation.

---

### Dependencies
The crate depends on several libraries, including:
- `serde` and `serde_json` for JSON handling.
- `tokio` for asynchronous runtime (optional).
- `reqwest` for HTTP requests.
- `tungstenite` for WebSocket support.
- `clap` for CLI argument parsing.

---

### Constants
- **`VERSION`**: Current version of the MCPR crate.

---
The **`Client`** struct in the `mcpr` crate provides a high-level interface for interacting with MCP servers. Below is a detailed summary of its functionality and methods:

---

### Struct Definition
```rust
pub struct Client<T: Transport> { /* private fields */ }
```
The `Client` struct is generic over a transport type `T` that implements the `Transport` trait.

---

### Methods

#### 1. **`new`**
```rust
pub fn new(transport: T) -> Self
```
- **Description**: Creates a new MCP client with the specified transport.
- **Parameters**: 
  - `transport`: An instance of a type implementing the `Transport` trait.
- **Returns**: A new `Client` instance.

#### 2. **`initialize`**
```rust
pub fn initialize(&mut self) -> Result<Value, MCPError>
```
- **Description**: Initializes the client, typically used to establish communication with the server.
- **Returns**: 
  - `Ok(Value)`: A JSON value containing server information or initialization data.
  - `Err(MCPError)`: An error if initialization fails.

#### 3. **`call_tool`**
```rust
pub fn call_tool<P: Serialize, R: DeserializeOwned>(
    &mut self, 
    tool_name: &str, 
    params: &P
) -> Result<R, MCPError>
```
- **Description**: Calls a tool on the server with the specified parameters.
- **Parameters**:
  - `tool_name`: The name of the tool to call.
  - `params`: Parameters for the tool, which must implement the `Serialize` trait.
- **Returns**:
  - `Ok(R)`: The response from the server, deserialized into the specified type `R`.
  - `Err(MCPError)`: An error if the tool call fails.

#### 4. **`shutdown`**
```rust
pub fn shutdown(&mut self) -> Result<(), MCPError>
```
- **Description**: Shuts down the client, typically used to clean up resources or close the connection.
- **Returns**:
  - `Ok(())`: If the shutdown is successful.
  - `Err(MCPError)`: An error if the shutdown fails.

---

### Example Usage
Here is an example of how to use the `Client` struct:

```rust
use mcpr::{
    client::Client,
    transport::stdio::StdioTransport,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with stdio transport
    let transport = StdioTransport::new();
    let mut client = Client::new(transport);

    // Initialize the client
    client.initialize()?;

    // Call a tool on the server
    let request = json!({
        "param1": "value1",
        "param2": "value2"
    });
    let response: serde_json::Value = client.call_tool("my_tool", &request)?;

    println!("Response: {:?}", response);

    // Shutdown the client
    client.shutdown()?;

    Ok(())
}
```

---

### Auto Trait Implementations
The `Client` struct automatically implements the following traits:
- **`Send`**: Safe to transfer across threads.
- **`Sync`**: Safe to share references across threads.
- **`Unpin`**: Can be safely moved in memory.
- **`RefUnwindSafe`** and **`UnwindSafe`**: Safe to use across unwind boundaries.

---

### Blanket Implementations
The `Client` struct also benefits from Rust's blanket implementations, such as:
- **`From<T>`**: Converts from one type to another.
- **`TryFrom<U>`** and **`TryInto<U>`**: For fallible conversions.
- **`Borrow<T>`** and **`BorrowMut<T>`**: For borrowing references.
- **`WithSubscriber`**: For attaching tracing subscribers.

---
The **`MCPError`** enum in the `mcpr` crate represents errors that can occur while using the Model Context Protocol (MCP). Below is a detailed summary of its structure, variants, and implementations:

---

### Enum Definition
```rust
pub enum MCPError {
    Serialization(Error),
    Transport(String),
    Protocol(String),
    UnsupportedFeature(String),
}
```

---

### Variants

1. **`Serialization(Error)`**
   - Represents errors related to serialization or deserialization of data.
   - Typically occurs when converting data to or from JSON or other formats.

2. **`Transport(String)`**
   - Represents errors in the transport layer.
   - The associated `String` provides details about the transport error.

3. **`Protocol(String)`**
   - Represents protocol-level errors.
   - The associated `String` provides details about the protocol violation or issue.

4. **`UnsupportedFeature(String)`**
   - Represents errors when attempting to use a feature that is not supported.
   - The associated `String` describes the unsupported feature.

---

### Trait Implementations

#### 1. **`Debug`**
   - Provides a debug representation of the error.

#### 2. **`Display`**
   - Formats the error as a human-readable string.

#### 3. **`Error`**
   - Implements the standard `Error` trait, allowing `MCPError` to be used with Rust's error-handling ecosystem.
   - Includes methods like:
     - `source()`: Returns the underlying cause of the error, if any.
     - Deprecated methods like `description()` and `cause()`.

#### 4. **`From<Error>`**
   - Allows conversion from a generic `Error` into an `MCPError` of the `Serialization` variant.

---

### Auto Trait Implementations

- **`Send`**: Safe to transfer across threads.
- **`Sync`**: Safe to share references across threads.
- **`Unpin`**: Can be safely moved in memory.
- **`Freeze`**: Immutable once created.
- **`!RefUnwindSafe`** and **`!UnwindSafe`**: Not safe to use across unwind boundaries.

---

### Blanket Implementations
The `MCPError` enum benefits from Rust's blanket implementations, such as:
- **`ToString`**: Converts the error to a string using its `Display` implementation.
- **`From<T>`**: Converts from one type to another.
- **`TryFrom<U>`** and **`TryInto<U>`**: For fallible conversions.
- **`Borrow<T>`** and **`BorrowMut<T>`**: For borrowing references.
- **`WithSubscriber`**: For attaching tracing subscribers.

---

### Example Usage

Here is an example of how to handle `MCPError` in a function:

```rust
use mcpr::error::MCPError;

fn process_data() -> Result<(), MCPError> {
    // Simulate a protocol error
    let error_message = "Invalid protocol message".to_string();
    Err(MCPError::Protocol(error_message))
}

fn main() {
    match process_data() {
        Ok(_) => println!("Data processed successfully"),
        Err(e) => match e {
            MCPError::Serialization(err) => println!("Serialization error: {:?}", err),
            MCPError::Transport(msg) => println!("Transport error: {}", msg),
            MCPError::Protocol(msg) => println!("Protocol error: {}", msg),
            MCPError::UnsupportedFeature(msg) => println!("Unsupported feature: {}", msg),
        },
    }
}
```

---
mcpr::generator
Function generate_clientCopy 

Summary
pub fn generate_client(
    name: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError>

mcpr::generator
Function generate_projectCopy 

pub fn generate_project(
    name: &str,
    output_dir: &str,
    transport_type: &str,
) -> Result<(), GeneratorError>

mcpr::generator
Function generate_serverCopy 

pub fn generate_server(
    name: &str,
    output_dir: &Path,
) -> Result<(), GeneratorError>

mcpr::generator
Enum GeneratorErrorCopy 

pub enum GeneratorError {
    Io(Error),
    Template(String),
    InvalidName(String),
}

The **`mcpr::schema::client`** module defines client-specific schema types for the Model Context Protocol (MCP). These types are used for communication between the client and the server, including requests, responses, notifications, and parameters. Below is a detailed summary of the module's contents:

---

### Overview
This module provides schema types that the client uses to interact with the server. These include:
- **Requests**: Sent by the client to the server to perform specific actions.
- **Responses**: Sent by the server in response to client requests.
- **Notifications**: Sent by either the client or the server to provide updates or information.
- **Parameters**: Used to define the structure of request and notification payloads.

---

### Structs

#### Key Structs
1. **`CallToolRequest`**
   - Used by the client to invoke a tool provided by the server.

2. **`InitializeRequest`**
   - Sent from the client to the server when it first connects to initialize the session.

3. **`ListToolsRequest`**
   - Sent by the client to request a list of tools available on the server.

4. **`ListToolsResult`**
   - The server's response to a `ListToolsRequest`, containing the list of tools.

5. **`GetPromptRequest`**
   - Used by the client to request a specific prompt from the server.

6. **`GetPromptResult`**
   - The server's response to a `GetPromptRequest`, containing the requested prompt.

7. **`ReadResourceRequest`**
   - Sent by the client to read a specific resource from the server.

8. **`ReadResourceResult`**
   - The server's response to a `ReadResourceRequest`, containing the resource content.

9. **`ProgressNotification`**
   - Used to inform the receiver of progress updates for long-running requests.

10. **`PingRequest`**
    - A ping message sent by either the client or the server to check if the other party is still alive.

---

#### Other Structs
- **`ArgumentInfo`**: Provides argument information for tool completion.
- **`ClientCapabilities`**: Describes the capabilities of the client.
- **`ListPromptsRequest`**: Requests a list of prompts and templates from the server.
- **`ListPromptsResult`**: The server's response to a `ListPromptsRequest`.
- **`ListResourcesRequest`**: Requests a list of resources from the server.
- **`ListResourcesResult`**: The server's response to a `ListResourcesRequest`.
- **`SetLevelRequest`**: Adjusts logging levels on the server.
- **`SubscribeRequest`**: Requests notifications for resource updates.
- **`UnsubscribeRequest`**: Cancels resource update notifications.

---

### Enums

1. **`Reference`**
   - Represents a reference to a prompt or resource.

2. **`ResourceContent`**
   - Represents the content of a resource.

---

### Example Usage

Here is an example of how a client might use some of these schema types:

```rust
use mcpr::schema::client::{InitializeRequest, CallToolRequest, ListToolsRequest};

fn main() {
    // Example: Initialize a client session
    let init_request = InitializeRequest {
        client_name: "ExampleClient".to_string(),
        client_version: "1.0.0".to_string(),
        capabilities: None, // Optional client capabilities
    };

    // Example: Request a list of tools
    let list_tools_request = ListToolsRequest {
        pagination: None, // Optional pagination parameters
    };

    // Example: Call a tool
    let call_tool_request = CallToolRequest {
        tool_name: "example_tool".to_string(),
        parameters: serde_json::json!({
            "param1": "value1",
            "param2": "value2"
        }),
    };

    // These requests would typically be sent to the server using an MCP client.
}
```

---

The **`mcpr::schema::common`** module defines common types used throughout the Model Context Protocol (MCP) schema. These types are shared across various parts of the protocol and include structures, enumerations, and type aliases for handling resources, prompts, tools, and more.

---

### Overview
This module provides foundational types that are used across the MCP schema, including:
- **Structs**: Representing resources, prompts, tools, and other entities.
- **Enums**: Representing logging levels, roles, and resource contents.
- **Type Aliases**: Simplifying commonly used types like cursors for pagination.

---

### Structs

#### Key Structs
1. **`Prompt`**
   - Represents a prompt or prompt template that the server offers.

2. **`Tool`**
   - Defines a tool that the client can call.

3. **`ToolInputSchema`**
   - Provides a JSON schema for tool input, ensuring that tool parameters are validated.

4. **`Resource`**
   - Represents a known resource that the server can read.

5. **`ResourceTemplate`**
   - Describes a template for resources available on the server.

6. **`TextContent`**
   - Represents text provided to or from a language model.

7. **`ImageContent`**
   - Represents an image provided to or from a language model.

8. **`EmbeddedResource`**
   - Represents the contents of a resource embedded into a prompt or tool call result.

9. **`Annotations`**
   - Provides optional annotations for objects, such as metadata or additional information.

10. **`Implementation`**
    - Describes an implementation of the MCP protocol.

---

#### Other Structs
- **`PromptArgument`**: Describes an argument that a prompt can accept.
- **`PromptMessage`**: Represents a message returned as part of a prompt.
- **`Root`**: Represents a root directory or file that the server can operate on.
- **`BlobResourceContents`**: Represents binary resource contents.
- **`TextResourceContents`**: Represents text resource contents.

---

### Enums

1. **`LoggingLevel`**
   - Represents the severity of a log message (e.g., `Info`, `Warning`, `Error`).

2. **`ProgressToken`**
   - A token used to associate progress notifications with the original request.

3. **`PromptMessageContent`**
   - Represents the content of a prompt message.

4. **`ResourceContents`**
   - Represents the contents of a specific resource or sub-resource.

5. **`Role`**
   - Represents the sender or recipient of messages and data in a conversation (e.g., `User`, `Assistant`).

---

### Type Aliases

1. **`Cursor`**
   - An opaque token used to represent a cursor for pagination.

---

### Example Usage

Here is an example of how some of these types might be used:

```rust
use mcpr::schema::common::{Prompt, Tool, ToolInputSchema, Resource};

fn main() {
    // Define a tool
    let tool = Tool {
        name: "example_tool".to_string(),
        description: Some("An example tool".to_string()),
        input_schema: ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some(serde_json::json!({
                "param1": {
                    "type": "string",
                    "description": "The first parameter"
                },
                "param2": {
                    "type": "integer",
                    "description": "The second parameter"
                }
            })),
            required: Some(vec!["param1".to_string()]),
        },
    };

    // Define a prompt
    let prompt = Prompt {
        name: "example_prompt".to_string(),
        description: Some("An example prompt".to_string()),
        arguments: vec![],
        messages: vec![],
    };

    // Define a resource
    let resource = Resource {
        uri: "example://resource".to_string(),
        description: Some("An example resource".to_string()),
    };

    println!("Tool: {:?}", tool);
    println!("Prompt: {:?}", prompt);
    println!("Resource: {:?}", resource);
}
```

---

The **`mcpr::schema::json_rpc`** module defines JSON-RPC message types for the Model Context Protocol (MCP). These types are used to structure requests, responses, notifications, and errors in a JSON-RPC-based communication system.

---

### Overview
This module provides the schema for JSON-RPC messages, including:
- **Requests**: Messages sent by the client to the server that expect a response.
- **Responses**: Messages sent by the server in response to client requests.
- **Notifications**: Messages that do not expect a response.
- **Errors**: Messages indicating an error occurred during processing.

---

### Modules

1. **`error_codes`**
   - Contains standard JSON-RPC error codes.

---

### Structs

#### Key Structs
1. **`JSONRPCRequest`**
   - Represents a request that expects a response.
   - Includes fields for the method name, parameters, and request ID.

2. **`JSONRPCResponse`**
   - Represents a successful (non-error) response to a request.
   - Includes fields for the result and the request ID.

3. **`JSONRPCError`**
   - Represents a response to a request that indicates an error occurred.
   - Includes an error object and the request ID.

4. **`JSONRPCNotification`**
   - Represents a notification that does not expect a response.
   - Includes fields for the method name and parameters.

5. **`JSONRPCErrorObject`**
   - Represents the error object in a JSON-RPC error response.
   - Includes fields for the error code, message, and optional data.

6. **`RequestParams`**
   - Represents the parameters for a request.

7. **`RequestMeta`**
   - Represents metadata for a request.

8. **`NotificationParams`**
   - Represents the parameters for a notification.

9. **`ResultBase`**
   - Represents the base structure for a result.

10. **`RequestBase`**
    - Represents the base structure for a request.

11. **`NotificationBase`**
    - Represents the base structure for a notification.

---

### Enums

1. **`JSONRPCMessage`**
   - Represents the different types of JSON-RPC messages, such as requests, responses, and notifications.

2. **`RequestId`**
   - Represents a uniquely identifying ID for a request in JSON-RPC.

---

### Type Aliases

1. **`EmptyResult`**
   - Represents a response that indicates success but carries no data.

---

### Example Usage

Here is an example of how to use some of these types:

```rust
use mcpr::schema::json_rpc::{JSONRPCRequest, JSONRPCResponse, JSONRPCError, RequestId};
use serde_json::json;

fn main() {
    // Example: Create a JSON-RPC request
    let request = JSONRPCRequest {
        jsonrpc: "2.0".to_string(),
        method: "example_method".to_string(),
        params: Some(json!({
            "param1": "value1",
            "param2": "value2"
        })),
        id: RequestId::Number(1),
    };

    // Example: Create a JSON-RPC response
    let response = JSONRPCResponse {
        jsonrpc: "2.0".to_string(),
        result: json!({
            "result_key": "result_value"
        }),
        id: RequestId::Number(1),
    };

    // Example: Create a JSON-RPC error
    let error = JSONRPCError {
        jsonrpc: "2.0".to_string(),
        error: JSONRPCErrorObject {
            code: -32601, // Method not found
            message: "The method does not exist".to_string(),
            data: None,
        },
        id: Some(RequestId::Number(1)),
    };

    println!("Request: {:?}", request);
    println!("Response: {:?}", response);
    println!("Error: {:?}", error);
}
```

---

The **`mcpr::schema::server`** module defines server-specific schema types for the Model Context Protocol (MCP). These types are used by the server to interact with the client, including requests, responses, notifications, and capabilities.

---

### Overview
This module provides schema types that the server uses to communicate with the client. These include:
- **Requests**: Sent by the server to the client to perform specific actions.
- **Responses**: Sent by the client in response to server requests.
- **Notifications**: Sent by the server to inform the client of changes or updates.
- **Capabilities**: Descriptions of the server's features and supported operations.

---

### Structs

#### Key Structs
1. **`InitializeResult`**
   - Sent by the server in response to an initialization request from the client.

2. **`CallToolResult`**
   - Represents the server's response to a tool call.

3. **`CompleteResult`**
   - Represents the server's response to a completion request.

4. **`CreateMessageRequest`**
   - A request from the server to the client to sample an LLM (Language Model).

5. **`CreateMessageResult`**
   - The client's response to a `CreateMessageRequest`.

6. **`ServerCapabilities`**
   - Describes the server's capabilities, such as supported tools, prompts, and resources.

7. **`LoggingMessageNotification`**
   - A notification sent by the server to the client, containing log messages.

8. **`PromptListChangedNotification`**
   - An optional notification informing the client that the list of prompts offered by the server has changed.

9. **`ResourceUpdatedNotification`**
   - A notification informing the client that a resource has been updated.

10. **`ToolListChangedNotification`**
    - An optional notification informing the client that the list of tools offered by the server has changed.

---

#### Other Structs
- **`CompletionInfo`**: Provides information about a completion request.
- **`ModelPreferences`**: Represents the server's preferences for model selection, requested from the client during sampling.
- **`ModelHint`**: Provides hints for model selection.
- **`ResourcesCapability`**: Describes the server's ability to handle resources.
- **`ToolsCapability`**: Describes the server's ability to handle tools.
- **`PromptsCapability`**: Describes the server's ability to handle prompts.

---

### Enums

1. **`IncludeContext`**
   - Represents options for including context in requests or responses.

2. **`KnownStopReason`**
   - Represents known reasons for stopping a process or request.

3. **`StopReason`**
   - Represents a reason for stopping a process, including custom reasons.

4. **`MessageContent`**
   - Represents the content of a message sent to or received from the client.

5. **`ToolResultContent`**
   - Represents the content of a tool's result.

---

### Example Usage

Here is an example of how the server might use some of these schema types:

```rust
use mcpr::schema::server::{InitializeResult, CallToolResult, ServerCapabilities};

fn main() {
    // Example: Server's response to an initialization request
    let init_result = InitializeResult {
        server_name: "ExampleServer".to_string(),
        server_version: "1.0.0".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(vec!["example_tool".to_string()]),
            prompts: Some(vec!["example_prompt".to_string()]),
            resources: Some(vec!["example_resource".to_string()]),
        },
    };

    // Example: Server's response to a tool call
    let tool_result = CallToolResult {
        tool_name: "example_tool".to_string(),
        result: serde_json::json!({
            "output": "Tool executed successfully"
        }),
    };

    println!("Initialization Result: {:?}", init_result);
    println!("Tool Call Result: {:?}", tool_result);
}
```

---
Module transportCopy item path
Settings
Help

Summary
Source
Transport layer for MCP communication

This module provides transport implementations for the Model Context Protocol (MCP). Transports handle the underlying mechanics of how messages are sent and received.

The following transport types are supported:

Stdio: Standard input/output for local processes
SSE: Server-Sent Events for server-to-client messages with HTTP POST for client-to-server
The following transport types are planned but not yet implemented:

WebSocket: Bidirectional communication over WebSockets (TBD)
Note: There are some linter errors related to async/await in this file. These errors occur because the async implementations require proper async HTTP and WebSocket clients. To fix these errors, you would need to:

Add proper async dependencies to your Cargo.toml
Implement the async methods using those dependencies
Use proper async/await syntax throughout the implementation
For now, the synchronous implementations are provided and work correctly.



The **`SSETransport`** struct in the `mcpr` crate provides a Server-Sent Events (SSE) transport implementation for the Model Context Protocol (MCP). It facilitates communication between the client and server using the SSE protocol.

---

### Overview
The `SSETransport` struct is used to send and receive messages over an SSE connection. It supports both client and server modes and implements the `Transport` trait, which defines methods for starting, sending, receiving, and closing connections.

---

### Struct Definition
```rust
pub struct SSETransport { /* private fields */ }
```

---

### Methods

#### 1. **`new`**
```rust
pub fn new(uri: &str) -> Self
```
- **Description**: Creates a new SSE transport in client mode.
- **Parameters**:
  - `uri`: The URI of the SSE endpoint.
- **Returns**: A new `SSETransport` instance.

#### 2. **`new_server`**
```rust
pub fn new_server(uri: &str) -> Self
```
- **Description**: Creates a new SSE transport in server mode.
- **Parameters**:
  - `uri`: The URI of the SSE endpoint.
- **Returns**: A new `SSETransport` instance configured for server mode.

---

### Trait Implementations

The `SSETransport` struct implements the `Transport` trait, which provides the following methods:

#### 1. **`start`**
```rust
fn start(&mut self) -> Result<(), MCPError>
```
- **Description**: Starts processing messages over the SSE connection.

#### 2. **`send`**
```rust
fn send<T: Serialize>(&mut self, message: &T) -> Result<(), MCPError>
```
- **Description**: Sends a message over the SSE connection.
- **Parameters**:
  - `message`: The message to send, which must implement the `Serialize` trait.

#### 3. **`receive`**
```rust
fn receive<T: DeserializeOwned>(&mut self) -> Result<T, MCPError>
```
- **Description**: Receives a message from the SSE connection.
- **Returns**: The received message, deserialized into the specified type.

#### 4. **`close`**
```rust
fn close(&mut self) -> Result<(), MCPError>
```
- **Description**: Closes the SSE connection.

#### 5. **`set_on_close`**
```rust
fn set_on_close(&mut self, callback: Option<CloseCallback>)
```
- **Description**: Sets a callback to be executed when the connection is closed.

#### 6. **`set_on_error`**
```rust
fn set_on_error(&mut self, callback: Option<ErrorCallback>)
```
- **Description**: Sets a callback to be executed when an error occurs.

#### 7. **`set_on_message`**
```rust
fn set_on_message<F>(&mut self, callback: Option<F>)
where
    F: Fn(&str) + Send + Sync + 'static,
```
- **Description**: Sets a callback to be executed when a message is received.
- **Parameters**:
  - `callback`: A function to handle incoming messages.

---

### Auto Trait Implementations

The `SSETransport` struct automatically implements the following traits:
- **`Send`**: Safe to transfer across threads.
- **`Sync`**: Safe to share references across threads.
- **`Unpin`**: Can be safely moved in memory.
- **`Freeze`**: Immutable once created.
- **`!RefUnwindSafe`** and **`!UnwindSafe`**: Not safe to use across unwind boundaries.

---

### Example Usage

Here is an example of how to use the `SSETransport` struct:

```rust
use mcpr::transport::sse::SSETransport;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new SSE transport in client mode
    let mut transport = SSETransport::new("http://example.com/sse");

    // Start the transport
    transport.start()?;

    // Send a message
    let message = json!({
        "type": "example",
        "content": "Hello, SSE!"
    });
    transport.send(&message)?;

    // Receive a message
    let received: serde_json::Value = transport.receive()?;
    println!("Received: {:?}", received);

    // Close the transport
    transport.close()?;

    Ok(())
}
```

---


The **`StdioTransport`** struct in the `mcpr` crate provides a transport implementation using standard input (stdin) and standard output (stdout). It is designed for communication between the client and server over standard I/O streams.

---

### Overview
The `StdioTransport` struct is a simple transport mechanism that uses stdin and stdout for message exchange. It implements the `Transport` trait, which provides methods for starting, sending, receiving, and closing connections.

---

### Struct Definition
```rust
pub struct StdioTransport { /* private fields */ }
```

---

### Methods

#### 1. **`new`**
```rust
pub fn new() -> Self
```
- **Description**: Creates a new `StdioTransport` instance using the default stdin and stdout.
- **Returns**: A new `StdioTransport` instance.

#### 2. **`with_reader_writer`**
```rust
pub fn with_reader_writer(
    reader: Box<dyn Read + Send>, 
    writer: Box<dyn Write + Send>
) -> Self
```
- **Description**: Creates a new `StdioTransport` instance with custom reader and writer streams.
- **Parameters**:
  - `reader`: A boxed reader implementing the `Read` and `Send` traits.
  - `writer`: A boxed writer implementing the `Write` and `Send` traits.
- **Returns**: A new `StdioTransport` instance configured with the provided reader and writer.

---

### Trait Implementations

The `StdioTransport` struct implements the following traits:

#### 1. **`Default`**
```rust
fn default() -> Self
```
- **Description**: Returns a default instance of `StdioTransport` using stdin and stdout.

#### 2. **`Transport`**
The `Transport` trait provides the following methods:

- **`start`**
  ```rust
  fn start(&mut self) -> Result<(), MCPError>
  ```
  - Starts processing messages over the standard I/O streams.

- **`send`**
  ```rust
  fn send<T: Serialize>(&mut self, message: &T) -> Result<(), MCPError>
  ```
  - Sends a message over the standard output stream.
  - **Parameters**:
    - `message`: The message to send, which must implement the `Serialize` trait.

- **`receive`**
  ```rust
  fn receive<T: DeserializeOwned>(&mut self) -> Result<T, MCPError>
  ```
  - Receives a message from the standard input stream.
  - **Returns**: The received message, deserialized into the specified type.

- **`close`**
  ```rust
  fn close(&mut self) -> Result<(), MCPError>
  ```
  - Closes the standard I/O streams.

- **`set_on_close`**
  ```rust
  fn set_on_close(&mut self, callback: Option<CloseCallback>)
  ```
  - Sets a callback to be executed when the connection is closed.

- **`set_on_error`**
  ```rust
  fn set_on_error(&mut self, callback: Option<ErrorCallback>)
  ```
  - Sets a callback to be executed when an error occurs.

- **`set_on_message`**
  ```rust
  fn set_on_message<F>(&mut self, callback: Option<F>)
  where
      F: Fn(&str) + Send + Sync + 'static,
  ```
  - Sets a callback to be executed when a message is received.
  - **Parameters**:
    - `callback`: A function to handle incoming messages.

---

### Auto Trait Implementations

The `StdioTransport` struct automatically implements the following traits:
- **`Send`**: Safe to transfer across threads.
- **`Unpin`**: Can be safely moved in memory.
- **`Freeze`**: Immutable once created.
- **`!Sync`**: Not safe to share references across threads.
- **`!RefUnwindSafe`** and **`!UnwindSafe`**: Not safe to use across unwind boundaries.

---

### Example Usage

Here is an example of how to use the `StdioTransport` struct:

```rust
use mcpr::transport::stdio::StdioTransport;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new StdioTransport instance
    let mut transport = StdioTransport::new();

    // Start the transport
    transport.start()?;

    // Send a message
    let message = json!({
        "type": "example",
        "content": "Hello, Stdio!"
    });
    transport.send(&message)?;

    // Receive a message
    let received: serde_json::Value = transport.receive()?;
    println!("Received: {:?}", received);

    // Close the transport
    transport.close()?;

    Ok(())
}
```

---
mcpr::transport
Trait TransportCopy item path
Settings
Help

Summary
Source
pub trait Transport {
    // Required methods
    fn start(&mut self) -> Result<(), MCPError>;
    fn send<T: Serialize>(&mut self, message: &T) -> Result<(), MCPError>;
    fn receive<T: DeserializeOwned>(&mut self) -> Result<T, MCPError>;
    fn close(&mut self) -> Result<(), MCPError>;
    fn set_on_close(&mut self, callback: Option<CloseCallback>);
    fn set_on_error(&mut self, callback: Option<ErrorCallback>);
    fn set_on_message<F>(&mut self, callback: Option<F>)
       where F: Fn(&str) + Send + Sync + 'static;
}


