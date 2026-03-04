# 📘 Hướng dẫn IMV — OpenFang Agent OS

> **IMV** = Internal Manual & Vận hành  
> Tài liệu hướng dẫn nội bộ cho đội ngũ vận hành và nhân viên sử dụng hệ thống OpenFang.

---

## 📑 Mục lục

- [Tổng quan](#-tổng-quan)
- [Kiến trúc hệ thống](#-kiến-trúc-hệ-thống)
- [Vai trò người dùng](#-vai-trò-người-dùng)
- [Hướng dẫn cho Admin](#-hướng-dẫn-cho-admin)
- [Hướng dẫn cho nhân viên](#-hướng-dẫn-cho-nhân-viên)
- [Agent — AI Agent là gì?](#-agent--ai-agent-là-gì)
- [Skills & Hands — Mở rộng năng lực Agent](#-skills--hands--mở-rộng-năng-lực-agent)
- [Channels — Kênh giao tiếp](#-channels--kênh-giao-tiếp)
- [Multi-Tenant — Quản lý nhiều workspace](#-multi-tenant--quản-lý-nhiều-workspace)
- [Ví dụ thực tế](#-ví-dụ-thực-tế)
- [Câu hỏi thường gặp (FAQ)](#-câu-hỏi-thường-gặp-faq)

---

## 🎯 Tổng quan

**OpenFang** là một **Agent OS** (Hệ điều hành AI Agent) — nền tảng cho phép tạo, quản lý và vận hành nhiều AI Agent đồng thời, phục vụ các tác vụ tự động hóa cho doanh nghiệp.

### Điểm nổi bật

| Tính năng | Mô tả |
|-----------|-------|
| 🤖 **Đa Agent** | Chạy nhiều AI Agent cùng lúc, mỗi agent có vai trò riêng |
| 📱 **Đa kênh** | Kết nối Zalo, Telegram, Slack, Email, WebChat,... |
| 🔌 **Plugin mở rộng** | Thêm Skills (Google Sheets, Drive, PDF,...) |
| 🏢 **Multi-Tenant** | Mỗi phòng ban / khách hàng có workspace riêng |
| 🔒 **Bảo mật** | RBAC, API Key, phân quyền theo vai trò |
| 💰 **Đa LLM** | Hỗ trợ Groq, OpenAI, Anthropic, Ollama, OpenRouter,... |

---

## 🏗️ Kiến trúc hệ thống

```
┌─────────────────────────────────────────────────────────────┐
│                     Admin Dashboard (:4200)                  │
│   Chat │ Agents │ Settings │ Channels │ Tenants │ Gallery   │
└────────────────────────────┬────────────────────────────────┘
                             │ HTTP API
┌────────────────────────────▼────────────────────────────────┐
│                      OpenFang Kernel                         │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Registry    │  │  Runtime    │  │  Channel Bridge     │ │
│  │  (quản lý    │  │ (gọi LLM,  │  │  (Zalo, Telegram,   │ │
│  │   agents)    │  │  xử lý)    │  │   Slack, Email,...) │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Memory     │  │  Skills     │  │  Hands (Tools)      │ │
│  │  (SQLite    │  │  (Plugins   │  │  (File, Web, Shell, │ │
│  │   lưu trữ)  │  │  mở rộng)  │  │   Code execution)   │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         ┌─────────┐   ┌─────────┐   ┌─────────┐
         │  Groq   │   │ OpenAI  │   │ Ollama  │
         │  (LLM)  │   │  (LLM)  │   │ (Local) │
         └─────────┘   └─────────┘   └─────────┘
```

### Luồng xử lý tin nhắn

```
 Nhân viên                OpenFang                    LLM Provider
    │                        │                             │
    │──── "Phân tích      ──→│                             │
    │      doanh thu Q1"     │                             │
    │                        │──── Gửi prompt + tools ───→│
    │                        │                             │
    │                        │◄─── "Cần đọc Sheets" ─────│
    │                        │                             │
    │                        │──── Gọi skill              │
    │                        │     google-sheets.read()   │
    │                        │                             │
    │                        │──── Gửi kết quả + prompt ─→│
    │                        │                             │
    │                        │◄─── "Báo cáo: ..." ───────│
    │                        │                             │
    │◄─── "📊 Báo cáo     ──│                             │
    │      doanh thu Q1..."  │                             │
```

---

## 👥 Vai trò người dùng

| Vai trò | Quyền hạn | Truy cập |
|---------|-----------|----------|
| **Super Admin** | Toàn quyền: quản lý Tenants, Agents, Channels, Settings | Dashboard `:4200` |
| **Admin** | Quản lý Agents, Channels trong tenant được gán | Dashboard `:4200` |
| **Member** | Chat với AI Agent, xem lịch sử | Dashboard hoặc Channel |
| **Nhân viên** | Chỉ chat với AI Agent qua kênh (Zalo, Email,...) | Zalo / Telegram / Email |

---

## 🔧 Hướng dẫn cho Admin

### 1. Truy cập Dashboard

```
http://<IP_SERVER>:4200
```

Đăng nhập bằng tài khoản admin (được cấu hình trong `config.toml`).

### 2. Tạo Agent mới

**Cách 1: Từ Gallery**
1. Vào **Agent Gallery** (sidebar)
2. Chọn template phù hợp (VD: `analyst`, `accountant`, `hr-assistant`)
3. Click **Deploy** → Agent tự động khởi tạo

**Cách 2: Từ file TOML**
1. Tạo thư mục trong `agents/`:
```
agents/
  └── my-agent/
      └── agent.toml
```

2. Nội dung `agent.toml`:
```toml
name = "my-agent"
description = "Trợ lý phân tích dữ liệu"
model = "llama-3.3-70b-versatile"
provider = "groq"

system_prompt = """
Bạn là chuyên gia phân tích dữ liệu kinh doanh.
Khi được hỏi, hãy đọc dữ liệu từ Google Sheets,
phân tích xu hướng, và viết báo cáo chuyên nghiệp.
"""

[skills]
allow = ["google-sheets", "google-drive", "email-reader"]

[tools]
allow = ["web_fetch", "file_read", "file_write", "code_run"]
```

3. Restart OpenFang hoặc hot-reload qua Dashboard.

### 3. Cấu hình Channel

Vào **Dashboard → Channels**, chọn kênh cần bật:

| Kênh | Setup |
|------|-------|
| **Zalo** | Lưu cookie từ chat.zalo.me vào file, cấu hình `cookie_path` |
| **Telegram** | Nhập Bot Token từ @BotFather |
| **Slack** | Nhập Bot Token + Signing Secret |
| **Email** | Nhập SMTP/IMAP credentials |
| **WebChat** | Tự động — truy cập `http://<IP>:4200` |

### 4. Quản lý Tenant

Vào **Dashboard → Tenants**:
- **Create Tenant** → tạo workspace mới
- Click tên tenant → xem chi tiết (Overview / Config / Members)
- **Add Member** → thêm thành viên vào tenant
- **Restart / Stop** → quản lý trạng thái

---

## 💬 Hướng dẫn cho nhân viên

### Nhân viên KHÔNG cần truy cập Dashboard

Nhân viên tương tác với AI Agent qua **kênh đã được Admin cấu hình sẵn** (Zalo, Telegram, Email,...).

### Cách sử dụng

**Bước 1:** Mở ứng dụng chat (Zalo / Telegram / Slack / Email)

**Bước 2:** Tìm bot/tài khoản AI đã được Admin cấu hình

**Bước 3:** Chat bằng tiếng Việt hoặc tiếng Anh, yêu cầu bất cứ điều gì:

```
📱 Zalo:
Bạn: "Đọc email mới nhất từ khách hàng ABC và tóm tắt cho tôi"
Bot: "📧 Có 3 email mới từ ABC Corp:
      1. Yêu cầu báo giá sản phẩm X (hôm nay 9:00)
      2. Xác nhận đơn hàng #1234 (hôm qua)
      3. Hỏi về chính sách bảo hành (2 ngày trước)
      Bạn muốn tôi trả lời email nào?"
```

```
📱 Telegram:
Bạn: "Phân tích file doanh thu Q1 trên Google Sheets"
Bot: "📊 Đã đọc file 'Doanh thu Q1 2026':
      - Tổng doanh thu: 2.5 tỷ VNĐ (+15% so với Q4)
      - Sản phẩm A: 1.2 tỷ (48%)
      - Sản phẩm B: 0.8 tỷ (32%)
      - Dịch vụ: 0.5 tỷ (20%)
      Xu hướng: Tăng trưởng tốt, đặc biệt SP A..."
```

```
📧 Email:
To: analyst@company-bot.com
Subject: Báo cáo tuần
Body: "Tạo báo cáo tuần từ dữ liệu Google Sheets,
       bao gồm biểu đồ doanh thu và nhận xét"

→ Bot tự động đọc Sheets, tạo báo cáo, upload lên Drive,
  và reply email kèm link báo cáo.
```

### Các yêu cầu nhân viên có thể gửi

| Loại yêu cầu | Ví dụ |
|---------------|-------|
| 📧 **Đọc email** | "Tóm tắt email chưa đọc" |
| 📊 **Phân tích Sheets** | "Phân tích doanh thu tháng này" |
| 📁 **Google Drive** | "Upload file báo cáo lên Drive" |
| 📝 **Viết báo cáo** | "Viết báo cáo tuần từ dữ liệu sales" |
| 🔍 **Tra cứu** | "Giá sản phẩm ABC là bao nhiêu?" |
| 📅 **Lịch hẹn** | "Lịch họp hôm nay có gì?" |
| 💡 **Tư vấn** | "Nên giảm giá bao nhiêu cho khách hàng VIP?" |
| 📋 **Tóm tắt** | "Tóm tắt cuộc họp hôm qua" |

---

## 🤖 Agent — AI Agent là gì?

### Khái niệm

Agent = 1 nhân viên AI chuyên trách, được cấu hình với:
- **Vai trò** (system prompt) — "Bạn là kế toán chuyên xử lý hóa đơn"
- **Mô hình AI** (model) — Groq, OpenAI, Ollama,...
- **Kỹ năng** (skills) — Google Sheets, Email, PDF,...
- **Công cụ** (tools/hands) — Đọc file, chạy code, gọi API,...

### Danh sách Agent có sẵn (51 templates)

| Agent | Vai trò |
|-------|---------|
| `assistant` | Trợ lý chung |
| `analyst` | Phân tích dữ liệu |
| `accountant` | Kế toán, xử lý hóa đơn |
| `copywriter` | Viết nội dung marketing |
| `hr-assistant` | Nhân sự, chính sách |
| `sales-agent` | Tư vấn bán hàng |
| `tech-support` | Hỗ trợ kỹ thuật |
| `translator` | Dịch thuật đa ngôn ngữ |
| `legal-advisor` | Tư vấn pháp lý |
| `data-engineer` | Xử lý dữ liệu, ETL |
| ... | [Xem đầy đủ trong Agent Gallery] |

### Multi-Agent (Đội nhóm AI)

Nhiều Agent có thể phối hợp:
```
Khách hàng hỏi ──→ sales-agent (tư vấn)
                        │
                        ├──→ analyst (tra giá, phân tích)
                        │
                        └──→ accountant (tạo báo giá)
```

---

## 🔌 Skills & Hands — Mở rộng năng lực Agent

### Hands = Công cụ tích hợp sẵn

| Tool | Chức năng | Ví dụ |
|------|-----------|-------|
| `web_fetch` | Đọc nội dung web | Tra giá, đọc tin tức |
| `file_read` | Đọc file nội bộ | Đọc PDF, CSV, Excel |
| `file_write` | Ghi file | Tạo báo cáo, xuất CSV |
| `shell_exec` | Chạy lệnh hệ thống | Script tự động |
| `code_run` | Chạy code Python/JS | Tính toán, phân tích |

### Skills = Plugin mở rộng (cài thêm)

| Skill | Chức năng | Cách hoạt động |
|-------|-----------|----------------|
| `google-sheets` | Đọc/ghi Google Sheets | Qua Google API |
| `google-drive` | Upload/download file | Qua Google API |
| `email-reader` | Đọc email IMAP | Kết nối mailbox |
| `email-sender` | Gửi email SMTP | Tự động reply |
| `pdf-reader` | Đọc file PDF | Trích xuất text |
| `calendar` | Quản lý lịch | Google Calendar API |
| `database` | Truy vấn database | SQL query |
| `webhook` | Gọi API bên ngoài | REST/GraphQL |

### Cách thêm skill cho Agent

Trong `agent.toml`:
```toml
[skills]
allow = ["google-sheets", "google-drive", "email-reader"]

[tools]
allow = ["web_fetch", "file_read", "code_run"]
# blocklist (không cho dùng):
block = ["shell_exec"]
```

---

## 📱 Channels — Kênh giao tiếp

### Danh sách kênh hỗ trợ (40+ kênh)

| Nhóm | Kênh |
|------|------|
| **Messaging** | Telegram, WhatsApp, Signal, Viber, **Zalo** |
| **Team Chat** | Slack, Discord, Teams, Mattermost, Rocket.Chat |
| **Email** | SMTP/IMAP (mọi provider) |
| **Social** | Messenger, Reddit, Bluesky, Mastodon, LinkedIn |
| **Enterprise** | Webex, Google Chat, Feishu/Lark, DingTalk |
| **Community** | Matrix, XMPP, IRC, Discourse, Mumble |
| **Asian** | LINE, **Zalo**, Kakao |
| **WebChat** | Tích hợp sẵn trong Dashboard |

### Cấu hình Zalo (kênh chính)

```toml
# config.toml
[channels.zalo]
cookie_path = "~/.openfang/zalo-cookie.txt"
```

**Cách lấy cookie Zalo:**
1. Mở trình duyệt → [chat.zalo.me](https://chat.zalo.me)
2. Đăng nhập bằng QR code
3. Bấm F12 → Console → gõ `document.cookie`
4. Copy toàn bộ chuỗi cookie
5. Dán vào file `~/.openfang/zalo-cookie.txt`
6. Vào Dashboard → Channels → Zalo → Save

---

## 🏢 Multi-Tenant — Quản lý nhiều workspace

### Khái niệm

Mỗi **Tenant** = 1 workspace độc lập với:
- Agents riêng
- Channels riêng
- Members riêng
- Quota riêng

### Plans & Quota

| Plan | Messages/ngày | Channels | Members | Giá |
|------|:------------:|:--------:|:-------:|-----|
| **Free** | 100 | 3 | 5 | Miễn phí |
| **Pro** | 1,000 | 10 | 20 | Liên hệ |
| **Enterprise** | Không giới hạn | Không giới hạn | Không giới hạn | Liên hệ |

### Quản lý Tenant

Truy cập: **Dashboard → Tenants** (hoặc `http://<IP>:4200/#tenants`)

| Thao tác | Cách làm |
|----------|----------|
| Tạo tenant | Click **Create Tenant** → nhập tên, chọn plan |
| Xem chi tiết | Click tên tenant → Overview / Config / Members |
| Thêm member | Tab Members → **Add Member** → nhập email |
| Dừng tenant | Click **Stop** → tiết kiệm tài nguyên |
| Khởi động lại | Click **Restart** |
| Xóa tenant | Click **Delete** (⚠️ không thể khôi phục) |

---

## 📋 Ví dụ thực tế

### Tình huống 1: Công ty bán hàng

```
Admin setup:
├── Tenant: "phong-kinh-doanh"
│   ├── Agent: sales-bot (tư vấn sản phẩm)
│   ├── Agent: lead-tracker (theo dõi khách hàng tiềm năng)
│   ├── Channel: Zalo (chat với khách)
│   └── Skills: google-sheets, email-reader
│
├── Tenant: "phong-ke-toan"
│   ├── Agent: accountant (xử lý hóa đơn)
│   ├── Agent: tax-agent (tính thuế)
│   ├── Channel: Email
│   └── Skills: google-sheets, pdf-reader
│
└── Tenant: "phong-nhan-su"
    ├── Agent: hr-bot (trả lời chính sách)
    ├── Agent: recruiter (sàng lọc CV)
    ├── Channel: Slack
    └── Skills: google-drive, email-reader
```

**Nhân viên KD chat Zalo:**
> "Khách hàng ABC hỏi giá sản phẩm X, số lượng 1000 cái"

**Bot trả lời:**
> "📋 Đã tra bảng giá:
> - Sản phẩm X, SL 1000: 50,000đ/cái (giảm 10% cho SL >500)
> - Tổng: 50,000,000 VNĐ
> - Thời gian giao: 5–7 ngày làm việc
> Bạn muốn tôi tạo báo giá chính thức không?"

### Tình huống 2: Tự động báo cáo

**Nhân viên gửi email:**
> To: analyst@bot.company.com  
> Subject: Báo cáo doanh thu tuần  
> Body: "Tạo báo cáo tuần từ sheet Doanh Thu 2026, gửi kết quả lên Drive thư mục Reports"

**Bot tự động:**
1. ✅ Đọc Google Sheets "Doanh Thu 2026"
2. ✅ Phân tích dữ liệu tuần hiện tại
3. ✅ Tạo file báo cáo (Markdown → PDF)
4. ✅ Upload lên Google Drive → `/Reports/Week-09-2026.pdf`
5. ✅ Reply email kèm link file

---

## ❓ Câu hỏi thường gặp (FAQ)

### Q: Nhân viên có cần cài phần mềm gì không?
**A:** Không. Nhân viên chỉ cần ứng dụng chat quen thuộc (Zalo, Telegram, Email,...).

### Q: AI có đọc được file PDF, Excel không?
**A:** Có, nếu Admin bật skill `pdf-reader` và `google-sheets` cho agent.

### Q: Dữ liệu có an toàn không?
**A:** Dữ liệu được xử lý trên server riêng, không gửi qua bên thứ 3 (trừ LLM provider). Dùng Ollama để chạy AI hoàn toàn nội bộ.

### Q: Có giới hạn số tin nhắn không?
**A:** Tùy plan:
- Free: 100 messages/ngày
- Pro: 1,000 messages/ngày
- Enterprise: Không giới hạn

### Q: Làm sao biết Agent đang xử lý?
**A:** Agent sẽ gửi emoji trạng thái:
- 🤔 Đang suy nghĩ
- ⚙️ Đang dùng tool
- ✍️ Đang viết câu trả lời
- ✅ Hoàn thành
- ❌ Lỗi

### Q: Có thể dùng nhiều AI model không?
**A:** Có. Mỗi Agent có thể dùng model khác nhau:
- Groq (miễn phí, nhanh): `llama-3.3-70b-versatile`
- OpenAI (chính xác): `gpt-4o`
- Ollama (chạy local): `qwen2.5:14b`
- OpenRouter (nhiều model free): `google/gemini-2.0-flash-exp:free`

---

## 🚀 Khởi động nhanh

```bash
# 1. Clone repo
git clone https://github.com/trongtri1512/openfang.git
cd openfang

# 2. Cấu hình
cp config.example.toml config.toml
# Sửa config.toml: thêm API key, cấu hình channels

# 3. Chạy Docker
docker compose up --build -d

# 4. Truy cập Dashboard
open http://localhost:4200
```

---

## 📞 Liên hệ hỗ trợ

| Kênh | Thông tin |
|------|-----------|
| GitHub | [github.com/trongtri1512/openfang](https://github.com/trongtri1512/openfang) |
| Email | admin@company.com |
| Zalo | Chat với bot hỗ trợ nội bộ |

---

> **Phiên bản:** OpenFang v0.3.4  
> **Cập nhật:** 2026-03-04  
> **Tác giả:** IMV Team
