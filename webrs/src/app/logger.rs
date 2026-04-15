use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init() {
    tracing_subscriber::registry()
        // 默认只打印 INFO 级别及以上的日志。
        // 但如果别人在终端输入 RUST_LOG=debug cargo run，它就会变成 debug 级别。
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true) // 打印是哪个文件报的错
                .with_line_number(true) // 打印报错在第几行
                .with_thread_ids(true) // 显示当前执行线程的 ID
                .with_thread_names(true) // 显示线程的名字
                .with_target(false), // 去掉完整路径
        )
        .init()
}
