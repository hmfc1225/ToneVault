# ToneVault

自托管的有声书管理平台，支持本地存储、WebDAV 和 RSS 多来源书库。

## 功能特性

- **多来源书库** — 本地目录、WebDAV 网盘、RSS 订阅，三种方式管理有声书
- **WebDAV 服务端** — 内置 WebDAV 服务器，支持远程访问和文件管理
- **音频流播放** — 支持 Range 请求的音频流，断点续播，播放进度自动保存
- **自动扫描** — 定时扫描书库目录，自动识别新书和元数据
- **元数据提取** — 自动从音频文件中提取标题、作者、封面等信息
- **用户认证** — JWT 认证，支持多用户和角色管理
- **暗色模式** — 全站暗色模式支持
- **响应式设计** — 适配桌面和移动端

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust · Axum · SQLx · SQLite |
| 前端 | Vue 3 · TypeScript · Tailwind CSS 4 |
| 认证 | JWT · Argon2 |
| 存储 | SQLite / PostgreSQL |

## 项目结构

```
ToneVault/
├── server/                        # Rust 后端
│   ├── tonevault-server/          #   Axum HTTP 服务，API 路由
│   ├── tonevault-core/            #   核心模型与业务逻辑
│   ├── tonevault-db/              #   数据库访问层 (SQLx)
│   ├── tonevault-auth/            #   认证与授权
│   └── tonevault-webdav/          #   WebDAV 服务端实现
├── web/                           # Vue 3 前端
│   └── src/
│       ├── views/                 #   页面组件
│       ├── components/            #   通用组件
│       ├── stores/                #   Pinia 状态管理
│       ├── api/                   #   API 客户端
│       └── types/                 #   TypeScript 类型定义
├── config/                        # 配置文件模板
├── Dockerfile                     # Docker 构建文件
└── docker-compose.yml             # Docker Compose 编排
```

## 快速开始

### Docker 部署（推荐）

```bash
# 克隆仓库
git clone https://github.com/hmfc1225/ToneVault.git
cd ToneVault

# 启动服务
docker compose up -d
```

服务启动后访问 `http://localhost:3000`，首次使用会引导创建管理员账号。

### 手动构建

#### 前置要求

- Rust 1.80+ 
- Node.js 20+
- npm 或 pnpm

#### 构建后端

```bash
cd server
cargo build --release
```

#### 构建前端

```bash
cd web
npm install
npm run build
```

#### 启动服务

```bash
# 复制配置文件
cp config/tonevault.example.toml config/tonevault.toml

# 启动后端（默认监听 0.0.0.0:3000）
cd server
cargo run --release

# 或开发模式
cd web
npm run dev    # 前端开发服务器
```

## 配置

配置文件位于 `config/tonevault.toml`，主要配置项：

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "sqlite:tonevault.db"

[auth]
secret = "your-jwt-secret-key"
token_expiry = "7d"
```

## API 概览

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/auth/login` | 用户登录 |
| POST | `/api/auth/setup` | 初始设置 |
| GET | `/api/libraries` | 获取书库列表 |
| POST | `/api/libraries` | 创建书库 |
| PUT | `/api/libraries/:id` | 更新书库 |
| DELETE | `/api/libraries/:id` | 删除书库 |
| POST | `/api/libraries/:id/scan` | 触发扫描 |
| POST | `/api/webdav/connect` | WebDAV 连接验证 |
| POST | `/api/webdav/list` | WebDAV 目录浏览 |
| GET | `/api/books` | 获取书籍列表 |
| GET | `/api/books/:id` | 获取书籍详情 |
| GET | `/api/books/:id/cover` | 获取封面图片 |
| GET | `/api/books/:id/tracks/:track/stream` | 音频流播放 |

## 书库来源

### 本地模式

直接读取服务器本地目录中的音频文件，支持自动扫描和文件监控。

```
/audiobooks/
├── 三体/
│   ├── 01.mp3
│   ├── 02.mp3
│   └── ...
└── 百年孤独/
    ├── 01.m4b
    └── ...
```

### WebDAV 模式

连接 WebDAV 网盘（如坚果云、NextCloud），浏览并选择目录作为书库。支持：
- 连接验证
- 目录浏览与选择
- 自动同步

### RSS 模式

订阅 RSS/Atom 播客源，自动获取和更新有声书内容。支持：
- RSS 2.0 和 Atom 格式
- 自动同步检查
- 新书自动入库

## 支持的音频格式

MP3, M4A, M4B, FLAC, OGG, WAV, AAC

## 开发

```bash
# 前端开发
cd web
npm install
npm run dev

# 后端开发
cd server
cargo run

# 类型检查
cd web && npx vue-tsc --noEmit

# 构建生产版本
cd web && npm run build
cd server && cargo build --release
```

## License

MIT
