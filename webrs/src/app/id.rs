use chrono::NaiveDate;
use idgenerator::{IdGeneratorOptions, IdInstance};

pub fn init() -> anyhow::Result<()> {
    let options = IdGeneratorOptions::new()
        .base_time(
            // 设置基准时间
            NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis(), // 最终转换成毫秒级的时间戳
        )
        // 设置机器码
        .worker_id(1)
        .worker_id_bit_len(4);

    Ok(IdInstance::init(options)?)
}

/// 注意返回值是 String，这是为了防止JavaScript在处理 64 位大整数时丢失精度
pub fn next_id() -> String {
    IdInstance::next_id().to_string()
}
