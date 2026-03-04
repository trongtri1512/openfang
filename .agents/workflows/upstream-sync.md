---
description: Hướng dẫn đồng bộ upstream OpenFang (RightNow-AI) về fork mà không ảnh hưởng tính năng Zalo và Tenants
---

# Upstream Sync Workflow — OpenFang

## 📋 Bản đồ xung đột (Conflict Risk Map)

### 🟢 File 100% Custom (KHÔNG BAO GIỜ bị upstream ghi đè)

Upstream **không có** những file này, nên **không bao giờ xung đột**:

| File | Tính năng |
|------|-----------|
| `openfang-api/src/tenants.rs` | Quản lý tenant (CRUD API) |
| `openfang-api/static/js/pages/tenants.js` | Frontend tenant tabs |
| `openfang-channels/src/zalo/mod.rs` | Zalo adapter core |
| `openfang-channels/src/zalo/auth.rs` | Zalo authentication |
| `openfang-channels/src/zalo/messaging.rs` | Zalo messaging |
| `openfang-api/src/channel_bridge.rs` | Channel bridge routes |

### 🟡 File chia sẻ — CẦN CẨN THẬN khi merge

Upstream **có thể sửa** các file này, và ta cũng đã sửa:

| File | Ta sửa gì | Upstream hay sửa gì | Mức rủi ro |
|------|-----------|---------------------|------------|
| `openfang-api/src/server.rs` | Thêm routes tenant/channel | Thêm routes mới | 🟡 Trung bình — khác vùng code |
| `openfang-api/src/lib.rs` | Import tenants module | Thêm modules mới | 🟢 Thấp |
| `openfang-api/static/index_body.html` | 7 tabs tenant detail | Bug fixes CSS, UI tweaks | 🔴 Cao — file lớn, nhiều thay đổi |
| `openfang-channels/src/lib.rs` | Thêm `pub mod zalo` | Thêm channels mới | 🟢 Thấp |

### 🟢 File upstream-only — KHÔNG RỦI RO

Ta không sửa, chỉ upstream sửa → apply trực tiếp:

| File | Upstream hay sửa |
|------|-----------------|
| `Cargo.toml` | Version bump |
| `model_catalog.rs` (runtime) | Thêm providers/models mới |
| `drivers/mod.rs` | Thêm provider defaults |
| `metering.rs` | Thêm cost rates |
| `hands/lib.rs` | Thêm fields/features |
| `openfang-cli/src/main.rs` | Bug fixes |

---

## 🔄 Quy trình Sync (Step-by-step)

### Bước 1: Kiểm tra upstream có gì mới

```bash
# Fetch upstream (không ảnh hưởng code local)
git fetch upstream

# Xem commits mới
git log --oneline main..upstream/main

# Xem files nào thay đổi
git diff --stat main..upstream/main
```

### Bước 2: Kiểm tra xung đột tiềm ẩn

```bash
# Xem upstream có sửa file nào ta đã custom không
git diff --name-only main..upstream/main | findstr /i "server.rs lib.rs index_body.html"
```

**Nếu kết quả trống:** An toàn, merge trực tiếp (Bước 3A)
**Nếu có file trùng:** Cần cherry-pick thủ công (Bước 3B)

### Bước 3A: Merge trực tiếp (nếu KHÔNG có xung đột)

```bash
git checkout -b sync-upstream
git merge upstream/main
# Nếu thành công:
git checkout main
git merge sync-upstream
git push origin main
```

### Bước 3B: Cherry-pick thủ công (nếu CÓ xung đột)

```bash
# 1. Tạo branch mới
git checkout -b sync-upstream

# 2. Xem diff từng file upstream thay đổi
git diff main..upstream/main -- crates/openfang-runtime/src/model_catalog.rs

# 3. Apply từng thay đổi bằng tay (hoặc nhờ AI)
# - Mở file, copy phần code mới từ upstream
# - Giữ nguyên code custom của mình

# 4. Commit + merge
git add -A
git commit -m "Sync upstream vX.Y.Z"
git checkout main
git merge sync-upstream
git push origin main
```

### Bước 4: Verify

```bash
docker compose up --build -d
# Kiểm tra dashboard, tenants, Zalo hoạt động bình thường
```

---

## ⚡ Mẹo quan trọng

1. **LUÔN tạo branch mới** trước khi sync — nếu hỏng thì `git checkout main` là quay lại an toàn
2. **KHÔNG dùng `git merge upstream/main` trực tiếp trên `main`** — luôn merge qua branch trung gian
3. **File `index_body.html` là RỦI RO CAO NHẤT** — upstream thường sửa file này, và ta cũng sửa nhiều
4. **Nếu dùng AI (tôi):** Chỉ cần nói "sync upstream" là tôi sẽ tự phân tích diff và apply an toàn
5. **Tần suất sync:** Nên sync mỗi 1-2 tuần để diff nhỏ, dễ merge hơn

## 🛡️ Tại sao Zalo và Tenants AN TOÀN?

- **Zalo adapter** nằm trong `openfang-channels/src/zalo/` — đây là thư mục **ta tự thêm**, upstream không có → **0% xung đột**
- **Tenants** nằm trong `tenants.rs` + `tenants.js` — đây là files **ta tự tạo**, upstream không có → **0% xung đột**
- **Tenant tabs HTML** nằm trong `index_body.html` — file này upstream CÓ sửa, nhưng thường chỉ sửa vài dòng CSS/JS nhỏ, còn code tenants của ta ở cuối file → **rủi ro thấp nếu cherry-pick cẩn thận**
