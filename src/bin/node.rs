use slog::{Logger, Drain, Level};
use tezedge_bootstrap_poc::Socket;

fn create_logger() -> Logger {
    let drain = slog_async::Async::new(
        slog_term::FullFormat::new(slog_term::TermDecorator::new().build())
            .build()
            .fuse(),
    )
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
