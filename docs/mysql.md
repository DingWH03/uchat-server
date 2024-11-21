## mysql

### **1. 用户表：`users`**

用于存储用户的基本信息。

| 字段名            | 类型          | 描述                 |
|-------------------|---------------|---------------------|
| `id`              | INT           | 主键，用户 ID        |
| `username`        | VARCHAR(50)   | 用户名               |
| `email`           | VARCHAR(100)  | 邮箱地址             |
| `password_hash`   | VARCHAR(255)  | 密码哈希             |
| `created_at`      | TIMESTAMP     | 注册时间             |
| `profile_picture` | VARCHAR(255)  | 用户头像 URL         |
| `status`          | VARCHAR(255)  | 状态信息（如“在线”） |

**示例 SQL**：

```sql
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    profile_picture VARCHAR(255),
    status VARCHAR(255)
);
```

### **2. 用户关系表：`user_relationships`**

用于存储用户之间的关系（如好友、关注）。

| 字段名       | 类型          | 描述                            |
|--------------|---------------|---------------------------------|
| `id`         | INT           | 主键                            |
| `user_id`    | INT           | 用户 ID（发起方）               |
| `friend_id`  | INT           | 用户 ID（接收方）               |
| `relationship_type` | ENUM('friend', 'follow', 'block') | 关系类型 |
| `created_at` | TIMESTAMP     | 关系创建时间                    |

**示例 SQL**：

```sql
CREATE TABLE user_relationships (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    friend_id INT NOT NULL,
    relationship_type ENUM('friend', 'follow', 'block') NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (friend_id) REFERENCES users(id)
);
```

### **3. 消息表：`messages`**

用于存储用户之间的消息。

| 字段名       | 类型          | 描述                          |
|--------------|---------------|-------------------------------|
| `id`         | BIGINT        | 主键，消息 ID                 |
| `sender_id`  | INT           | 发送方用户 ID                 |
| `receiver_id`| INT           | 接收方用户 ID                 |
| `content`    | TEXT          | 消息内容                      |
| `created_at` | TIMESTAMP     | 发送时间                      |
| `status`     | ENUM('sent', 'delivered', 'read') | 消息状态 |

**示例 SQL**：

```sql
CREATE TABLE messages (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    sender_id INT NOT NULL,
    receiver_id INT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status ENUM('sent', 'delivered', 'read') DEFAULT 'sent',
    FOREIGN KEY (sender_id) REFERENCES users(id),
    FOREIGN KEY (receiver_id) REFERENCES users(id)
);
```

### **4. 群组表（可选）：`groups` 和 `group_members`**

如果需要支持群聊，设计群组相关表：

- **`groups`**：存储群组基本信息。
- **`group_members`**：存储群组成员信息。

**示例 SQL**：

```sql
CREATE TABLE groups (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_by INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE group_members (
    id INT AUTO_INCREMENT PRIMARY KEY,
    group_id INT NOT NULL,
    user_id INT NOT NULL,
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

### **5. 好友列表**

你已经有了 `user_relationships` 表来存储用户与用户之间的关系。通过这个表，你可以通过以下查询获得某个用户的好友列表：

```sql
SELECT u.id, u.username, u.profile_picture, r.relationship_type
FROM users u
JOIN user_relationships r ON u.id = r.friend_id
WHERE r.user_id = ? AND r.relationship_type = 'friend';
```

### **6. 群聊列表**

同理，群聊信息可以通过 `group_members` 表来获取。每个群组成员都会在这个表中有一条记录，表示用户加入了某个群组。你可以通过以下查询来获取某个用户加入的所有群聊：

```sql
SELECT g.id, g.name, g.created_at
FROM groups g
JOIN group_members gm ON g.id = gm.group_id
WHERE gm.user_id = ?;
```
