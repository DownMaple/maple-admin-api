use rsa::{RsaPrivateKey, Pkcs1v15Encrypt};
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::PublicKeyParts;
use base64::{Engine as _, engine::general_purpose};
use super::error::AppError;
use super::key_manager::KeyManager;
use std::sync::Mutex;
use std::sync::OnceLock;

// 全局密钥管理器实例
static KEY_MANAGER: OnceLock<Mutex<Option<KeyManager>>> = OnceLock::new();

/// 初始化密钥管理器
pub fn init_key_manager() -> Result<(), AppError> {
    let manager = KeyManager::new()?;
    KEY_MANAGER.get_or_init(|| Mutex::new(Some(manager)));
    Ok(())
}

/// 获取密钥管理器
fn get_key_manager() -> Result<KeyManager, AppError> {
    let mutex = KEY_MANAGER.get()
        .ok_or_else(|| AppError::InternalServerError("密钥管理器未初始化".to_string()))?;
    
    let guard = mutex.lock()
        .map_err(|e| AppError::InternalServerError(format!("密钥管理器锁异常: {}", e)))?;
    
    guard.clone()
        .ok_or_else(|| AppError::InternalServerError("密钥管理器未初始化".to_string()))
}

/// 解密密码
pub fn decrypt_password(encrypted_base64: &str) -> Result<String, AppError> {
    let key_manager = get_key_manager()?;
    
    tracing::debug!("加密密码长度: {}", encrypted_base64.len());
    tracing::debug!("加密密码前 50 字符: {}", &encrypted_base64[..encrypted_base64.len().min(50)]);
    
    // 解析私钥 (PKCS#8 格式)
    let private_key = RsaPrivateKey::from_pkcs8_pem(key_manager.get_private_key())
        .map_err(|e| {
            tracing::error!("❌ 私钥解析失败: {}", e);
            AppError::InternalServerError(format!("私钥解析失败: {}", e))
        })?;
    
    tracing::debug!("✅ 私钥解析成功，密钥大小: {} bits", private_key.size() * 8);

    // Base64 解码
    let encrypted_data = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|e| {
            tracing::error!("❌ Base64 解码失败: {}", e);
            AppError::BadRequest(format!("密码Base64解码失败: {}", e))
        })?;
    
    tracing::debug!("✅ Base64 解码成功，加密数据长度: {} bytes", encrypted_data.len());

    // RSA 解密
    let decrypted_data = private_key
        .decrypt(Pkcs1v15Encrypt, &encrypted_data)
        .map_err(|e| {
            tracing::error!("❌ RSA 解密失败: {}", e);
            tracing::error!("   可能原因: 1) 前后端使用了不同的密钥对  2) 密码格式错误  3) 加密算法不匹配");
            AppError::BadRequest(format!("密码RSA解密失败，请检查密码格式: {}", e))
        })?;
    
    tracing::debug!("✅ RSA 解密成功，明文长度: {} bytes", decrypted_data.len());

    // 转换为字符串
    let plain_password = String::from_utf8(decrypted_data)
        .map_err(|e| {
            tracing::error!("❌ UTF-8 解码失败: {}", e);
            AppError::BadRequest(format!("密码UTF-8解码失败: {}", e))
        })?;
    
    tracing::debug!("✅ 密码解密完成，明文密码长度: {}", plain_password.len());
    
    Ok(plain_password)
}

/// 获取公钥
pub fn get_public_key() -> Result<String, AppError> {
    let key_manager = get_key_manager()?;
    Ok(key_manager.get_public_key().to_string())
}
