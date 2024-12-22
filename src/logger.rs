use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

fn get_subscriber(name: String, env_filter: &str) -> impl Subscriber + Sync + Send {
    Registry::default()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{name}={env_filter}").into()),
        )
        .with(fmt::Layer::default())
}

pub fn setup_logging(log_lvl: &str) {
    let package_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let subscriber = get_subscriber(package_name, log_lvl);
    set_global_default(subscriber).expect("Failed to set subscriber");
}
