use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
// 告诉 Serde 在解析数据时不要去找什么特定标签，而是直接按顺序去猜
#[serde(untagged)]
enum StringOrNumber<T> {
    String(String),
    Number(T),
}

pub fn deserialize_number<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + Deserialize<'de>,
    T::Err: Display,
    D: Deserializer<'de>,
{
    match StringOrNumber::deserialize(deserializer)? {
        // 如果解析成功，就接着往下看看是什么
        StringOrNumber::String(s) => s.parse().map_err(|e| serde::de::Error::custom(e)),
        StringOrNumber::Number(n) => Ok(n),
    }
}
