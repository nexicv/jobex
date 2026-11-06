//! `jobex-assistant serve [config.toml]` runs the proxy.
//! `jobex-assistant hash-pass` makes TEAM_PASS_HASH from a passphrase typed at a hidden prompt.
//! `jobex-assistant seal-key` prints a new random SEAL_KEY.

use anyhow::{Context, Result, bail};
use jobex_assistant::{chat, config, prompt, router};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("serve") | None => serve(args.get(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("config.toml"))).await,
        Some("hash-pass") => hash_pass(),
        Some("seal-key") => {
            let key: [u8; 32] = rand::random();
            println!("SEAL_KEY={}", key.iter().map(|b| format!("{b:02x}")).collect::<String>());
            Ok(())
        }
        Some(other) => bail!("unknown command {other:?}; use serve, hash-pass or seal-key"),
    }
}

fn hash_pass() -> Result<()> {
    let a = rpassword::prompt_password("Team passphrase: ")?;
    let b = rpassword::prompt_password("Same again: ")?;
    if a != b {
        bail!("the two entries differ; nothing was written");
    }
    println!("TEAM_PASS_HASH={}", jobex_assistant::team::hash_for_env(&a)?);
    Ok(())
}

async fn serve(config_path: PathBuf) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .init();
    let cfg = config::Config::load(&config_path)?;
    let secrets = config::Secrets::from_env()?;
    let assets = prompt::Assets::load(&cfg.assistant_dir).context("loading the assistant folder")?;
    let listen: SocketAddr = cfg.listen.parse().with_context(|| format!("bad listen address {:?}", cfg.listen))?;
    tracing::info!(
        model = %cfg.deepseek.model,
        effort = %cfg.deepseek.reasoning_effort,
        tools = assets.tools.as_array().map(Vec::len).unwrap_or(0),
        prompt_chars = assets.system_prompt.len(),
        team = secrets.team_pass_hash.is_some(),
        usd_per_hour = cfg.limits.usd_per_hour,
        "starting on {listen}"
    );
    let app = Arc::new(chat::App::new(cfg, assets, secrets)?);

    let sweeper = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(600)).await;
            sweeper.limiter.sweep(Instant::now());
        }
    });

    let listener = tokio::net::TcpListener::bind(listen).await?;
    axum::serve(listener, router(app).into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
