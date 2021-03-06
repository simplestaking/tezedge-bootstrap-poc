use slog::{Logger, Drain, Level};
use logging::file::FileAppenderBuilder;
use tezedge_bootstrap_poc::Socket;

fn create_logger() -> Logger {
    let _ = std::fs::remove_file("target/_.log");
    let appender = FileAppenderBuilder::new("target/_.log")
        .rotate_size(0x10000000) // 256 MB
        .build();
    let format = slog_term::FullFormat::new(slog_term::PlainDecorator::new(appender))
        .build()
        .fuse();
    let drain = slog_async::Async::new(format)
        .chan_size(32768)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .filter_level(Level::Debug)
        .fuse();

    Logger::root(drain, slog::o!())
}

#[tokio::main]
async fn main() {
    let logger = create_logger();
    let address = std::env::args().nth(1).unwrap();
    let (mut socket, _) = Socket::outgoing(address.parse().unwrap());
    socket.run(&logger).await.unwrap();
}
