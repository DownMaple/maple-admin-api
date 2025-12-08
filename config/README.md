# RSA 密钥配置说明

## 📁 目录结构

```
config/
├── README.md                 # 本文件
├── rsa_private_key.pem      # RSA 私钥（不要提交到 Git）
└── rsa_public_key.pem       # RSA 公钥
```

## 🔐 密钥加载优先级

系统按以下优先级加载 RSA 密钥：

1. **环境变量**（最高优先级）
   ```bash
   export RSA_PRIVATE_KEY="$(cat config/rsa_private_key.pem)"
   export RSA_PUBLIC_KEY="$(cat config/rsa_public_key.pem)"
   ```

2. **配置文件**
   - `config/rsa_private_key.pem`
   - `config/rsa_public_key.pem`

3. **内置密钥**（仅用于开发环境）
   - 代码中内置的默认密钥
   - ⚠️ 生产环境不要使用！

## 🛠️ 生成新的密钥对

### 方法1：使用 OpenSSL（推荐）

```bash
# 1. 生成私钥（2048位）
openssl genrsa -out config/rsa_private_key.pem 2048

# 2. 从私钥生成公钥
openssl rsa -in config/rsa_private_key.pem -pubout -out config/rsa_public_key.pem

# 3. 验证密钥对
openssl rsa -in config/rsa_private_key.pem -check
```

### 方法2：使用 OpenSSL（PKCS#8 格式）

```bash
# 1. 生成私钥
openssl genpkey -algorithm RSA -out config/rsa_private_key.pem -pkeyopt rsa_keygen_bits:2048

# 2. 从私钥生成公钥
openssl rsa -in config/rsa_private_key.pem -pubout -out config/rsa_public_key.pem
```

## 🚀 部署配置

### 开发环境

使用内置密钥或配置文件即可：

```bash
# 不需要额外配置，系统会使用内置密钥
cargo run
```

### 测试环境

使用配置文件：

```bash
# 1. 生成密钥对
openssl genrsa -out config/rsa_private_key.pem 2048
openssl rsa -in config/rsa_private_key.pem -pubout -out config/rsa_public_key.pem

# 2. 启动应用
cargo run
```

### 生产环境（推荐使用环境变量）

```bash
# 1. 生成密钥对
openssl genrsa -out /secure/path/rsa_private_key.pem 2048
openssl rsa -in /secure/path/rsa_private_key.pem -pubout -out /secure/path/rsa_public_key.pem

# 2. 设置环境变量
export RSA_PRIVATE_KEY="$(cat /secure/path/rsa_private_key.pem)"
export RSA_PUBLIC_KEY="$(cat /secure/path/rsa_public_key.pem)"

# 3. 启动应用
./maple-admin-api
```

### Docker 部署

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/maple-admin-api .

# 密钥通过环境变量传入
ENV RSA_PRIVATE_KEY=""
ENV RSA_PUBLIC_KEY=""

CMD ["./maple-admin-api"]
```

```bash
# 启动容器
docker run -d \
  -e RSA_PRIVATE_KEY="$(cat config/rsa_private_key.pem)" \
  -e RSA_PUBLIC_KEY="$(cat config/rsa_public_key.pem)" \
  -p 3000:3000 \
  maple-admin-api
```

### Kubernetes 部署

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: rsa-keys
type: Opaque
stringData:
  private_key: |
    -----BEGIN PRIVATE KEY-----
    ...
    -----END PRIVATE KEY-----
  public_key: |
    -----BEGIN PUBLIC KEY-----
    ...
    -----END PUBLIC KEY-----
```

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: maple-admin-api
spec:
  template:
    spec:
      containers:
      - name: api
        image: maple-admin-api:latest
        env:
        - name: RSA_PRIVATE_KEY
          valueFrom:
            secretKeyRef:
              name: rsa-keys
              key: private_key
        - name: RSA_PUBLIC_KEY
          valueFrom:
            secretKeyRef:
              name: rsa-keys
              key: public_key
```

## 🔒 安全建议

### 1. 文件权限

```bash
# 设置私钥文件权限（仅所有者可读）
chmod 600 config/rsa_private_key.pem

# 设置公钥文件权限
chmod 644 config/rsa_public_key.pem
```

### 2. Git 忽略

确保 `.gitignore` 包含：

```gitignore
# RSA 密钥
config/rsa_private_key.pem
config/rsa_public_key.pem
```

### 3. 密钥轮换

建议定期轮换密钥（如每 90 天）：

```bash
# 1. 生成新密钥
openssl genrsa -out config/rsa_private_key_new.pem 2048
openssl rsa -in config/rsa_private_key_new.pem -pubout -out config/rsa_public_key_new.pem

# 2. 备份旧密钥
mv config/rsa_private_key.pem config/rsa_private_key_old.pem
mv config/rsa_public_key.pem config/rsa_public_key_old.pem

# 3. 使用新密钥
mv config/rsa_private_key_new.pem config/rsa_private_key.pem
mv config/rsa_public_key_new.pem config/rsa_public_key.pem

# 4. 重启应用
```

## 🏢 密码机集成（未来支持）

系统预留了密码机接口，未来可以集成：

### 硬件密码机（HSM）

- 支持国密 SM2/SM3/SM4 算法
- 符合 GM/T 0018 标准
- 密钥不出密码机

### 云密钥管理服务（KMS）

- 阿里云 KMS
- 腾讯云 KMS
- AWS KMS
- Azure Key Vault

### 集成方式

```rust
// 未来实现示例
use crate::common::key_manager::{CryptoDeviceService, CryptoDeviceConfig};

let config = CryptoDeviceConfig {
    device_type: "kms".to_string(),
    endpoint: "https://kms.aliyuncs.com".to_string(),
    credentials: "your-credentials".to_string(),
    key_id: "your-key-id".to_string(),
};

let crypto_device = CryptoDeviceManager::new(config);
let decrypted = crypto_device.decrypt(&encrypted_data)?;
```

## 📚 相关文档

- [OpenSSL 文档](https://www.openssl.org/docs/)
- [RSA 加密算法](https://en.wikipedia.org/wiki/RSA_(cryptosystem))
- [PKCS#8 标准](https://tools.ietf.org/html/rfc5208)
