use pvr_core::config::Backend;
use smallvec::SmallVec;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::cli::Cli;

pub fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");
}
