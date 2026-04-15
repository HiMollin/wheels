use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::LazyLock;
use std::time::Duration;

const DEFAULT_SECRET: &str = "12345678";

static DEFAULT_JWT: LazyLock<JWT> = LazyLock::new(|| JWT::default());

// 用户的 ID 和名字
#[derive(Serialize, Debug, Clone)]
pub struct Principal {
    pub id: String,
    pub name: String,
}

/// JWT 标准 Payload
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    jti: String, // JWT ID: 唯一编号，通常用来防重放攻击（防止坏人拿旧 Token 重复请求）
    sub: String, // Subject (主题): 这个 Token 是发给谁的？（这里会存用户的 id 和 name）
    aud: String, // Audience (受众): 谁有资格接收这个 Token？
    iss: String, // Issuer (签发者): 谁发的这个 Token？
    iat: u64,    // Issued At: 签发时间（时间戳）
    exp: u64,    // Expiration Time: 过期时间（时间戳），一到这个时间自动失效
}

/// JWT 配置单
#[derive(Debug)]
pub struct JwtConfig {
    pub secret: Cow<'static, str>, // 密钥
    pub expiration: Duration,      // 有效期时长
    pub audience: String,          // 接收方标识
    pub issuer: String,            // 签发方标识
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: Cow::Borrowed(DEFAULT_SECRET),
            expiration: Duration::from_secs(60 * 60),
            audience: "audience".to_string(),
            issuer: "issuer".to_string(),
        }
    }
}

pub struct JWT {
    encode_secret: EncodingKey, // 加密用的钥匙
    decode_secret: DecodingKey, // 解密用的钥匙
    header: Header,             // JWT 的头部（声明使用的算法）
    validation: Validation,     // 验证规则（比如必须检查是否过期）
    expiration: Duration,       // 存活时长
    audience: String,           // 受众
    issuer: String,             // 签发者
}

impl JWT {
    pub fn new(config: JwtConfig) -> Self {
        // 创建验证规则，指定使用 HS256 对称加密算法
        let mut validation = Validation::new(Algorithm::HS256);

        // 告诉验证器：解密时，必须检查 audience 和 issuer 对不对得上
        validation.set_audience(&[&config.audience]);
        validation.set_issuer(&[&config.issuer]);

        // 要求 Token 里必须包含这 6 个标准字段
        validation.set_required_spec_claims(&["jti", "sub", "aud", "iss", "iat", "exp"]);

        // 把字符串密钥变成字节数组
        let secret = config.secret.as_bytes();

        Self {
            encode_secret: EncodingKey::from_secret(secret),
            decode_secret: DecodingKey::from_secret(secret),
            header: Header::new(Algorithm::HS256),
            validation: validation,
            expiration: config.expiration,
            audience: config.audience,
            issuer: config.issuer,
        }
    }

    /// 制证，登录成功时调用
    pub fn encode(&self, principal: Principal) -> anyhow::Result<String> {
        let current_timestamp = get_current_timestamp();

        let claims = Claims {
            jti: xid::new().to_string(), // 生成一个全局唯一的 ID 作为凭证号
            sub: format!("{}:{}", principal.id, principal.name),
            aud: self.audience.clone(),
            iss: self.issuer.clone(),
            iat: current_timestamp,
            // saturating_add 是防溢出的加法
            exp: current_timestamp.saturating_add(self.expiration.as_secs()),
        };

        // 调用底层的 encode 方法，把 header、claims 和钥匙放进去，生成最终的 Token 字符串！
        Ok(encode(&self.header, &claims, &self.encode_secret)?)
    }

    /// 每次带 Token 请求时调用
    pub fn decode(&self, token: &str) -> anyhow::Result<Principal> {
        // 调用底层的 decode，它会自动校验签名、过期时间、签发者等
        // 一旦过期或者被篡改，这一步直接报错抛出！如果成功，就拿到 claims
        let claims: Claims = decode(token, &self.decode_secret, &self.validation)?.claims;

        // 把刚才拼起来的 sub 拆开（按冒号拆分，最多拆成两份）
        let mut parts = claims.sub.splitn(2, ':');

        Ok(Principal {
            id: parts.next().unwrap().to_string(),
            name: parts.next().unwrap().to_string(),
        })
    }
}

impl Default for JWT {
    fn default() -> Self {
        Self::new(JwtConfig::default())
    }
}

pub fn get_jwt() -> &'static JWT {
    &DEFAULT_JWT
}
