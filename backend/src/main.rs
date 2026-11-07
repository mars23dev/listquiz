// ListQuiz backend entry point. All behaviour lives in the library so the
// integration tests under `tests/` can reach it.
use std::net::SocketAddr;
use std::path::PathBuf;

use listquiz::app_state::AppState;
use listquiz::db::{self, default_db_path};
use listquiz::modules::default_lists;
use listquiz::routes::api;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("listquiz: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = db_path_from_args();
    let conn = db::open_db(&db_path)?;
    db::seed_default_folder(&conn)?;
    default_lists::seed_default_lists(&conn, "default")?;

    let dist_dir = api::default_dist_dir();
    if api::frontend_built(&dist_dir) {
        println!(
            "listquiz: serving {} on http://0.0.0.0:8080 (open http://localhost:8080)",
            dist_dir.display()
        );
    } else {
        println!("listquiz: {}", api::missing_frontend_notice(&dist_dir));
    }

    // Every device in the room reaches the host over the local network, so the
    // server listens on all interfaces rather than loopback only.
    let addr: SocketAddr = ([0, 0, 0, 0], 8080).into();
    let state = AppState::new(conn, dist_dir);
    api::serve(state, addr, shutdown_signal()).await?;
    Ok(())
}

/// Resolve on Ctrl+C or, on Unix, on SIGTERM (the Docker stop signal). The
/// server then stops accepting requests and lets in-flight work finish before
/// the process exits, so SQLite closes cleanly.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// `--db <path>` overrides the default project-local database file.
fn db_path_from_args() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--db") {
        Some(index) => args
            .get(index + 1)
            .map(PathBuf::from)
            .unwrap_or_else(default_db_path),
        None => default_db_path(),
    }
}
