使用一个结构体clientsocket存储client的数据与状态，状态为枚举变量，每个clientsocket具有一个连接状态以及cookie，用于管理每个tcp连接的认证以及连接状态，旧的tcp连接销毁时保留clientsocket，下次重新连接时依然使用同一个对象，当客户端带着相同cookie连接时直接接入，此外若客户端带有空cookie或是cookie过期后要求重新连接，cookie过期时间为7天，每次连接时检测该cookie是否过期，若不断开tcp链接即不主动断开连接

1. **`ClientSocket` 结构体**：用来存储客户端的数据、连接状态和 cookie。
2. **连接状态枚举 `ConnectionStatus`**：表示客户端连接的状态。
3. **`clientsocket` 管理：** 使用一个全局的哈希表或集合来管理所有 `ClientSocket` 对象。
4. **cookie 过期检查**：在客户端连接时检查其 cookie 是否有效，若无 cookie 或 cookie 过期则要求重新连接。

### **解释**

1. **`ClientSocket` 结构体**：
   - 包含 `client_id`、`status`（连接状态）、`cookie`（用于认证）、`last_connected`（记录上次连接时间）。
   - `is_cookie_valid` 方法用来检查 cookie 是否有效（是否过期）。默认 cookie 过期时间为 7 天。
   - `renew_cookie` 用来为客户端生成新的 cookie。

2. **`ConnectionStatus` 枚举**：
   - `Connected`：客户端已连接。
   - `Disconnected`：客户端已断开连接。
   - `Reconnecting`：客户端正在尝试重新连接。

3. **`ClientManager` 结构体**：
   - 使用 `HashMap` 存储所有 `ClientSocket` 对象，键是 `cookie`，值是 `Arc<Mutex<ClientSocket>>`，确保线程安全。
   - `get_or_create_client` 方法根据传入的 cookie 查找已有的客户端，如果没有找到或 cookie 无效，则创建一个新的客户端。

4. **TCP 服务器**：
   - 使用 `TcpListener` 模拟服务器监听客户端的连接。每次连接会启动一个新线程来处理。

### **功能点**

- **自动管理客户端**：每个客户端连接时都会检查其 cookie 是否有效。如果有效则继续使用之前的 `ClientSocket` 对象，否则创建新的对象。
- **cookie 过期检查**：每次客户端连接时，会检查 cookie 是否过期（7 天）。过期后，要求重新连接。
- **保持连接状态**：如果客户端未断开连接，系统不会主动断开连接。

### **注意事项**

- **cookie 存储方式**：目前是直接生成一个新的 `UUID` 作为 cookie，你可以根据实际需求将其替换为实际的 cookie 或 session 管理机制。
- **并发处理**：此代码使用 `Arc<Mutex<T>>` 来确保在多线程环境中安全访问 `ClientSocket`。
