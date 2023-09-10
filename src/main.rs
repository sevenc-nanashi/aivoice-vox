#![allow(dead_code)]
mod aivoice;
mod bridge;
mod error;
mod icon_manager;
mod routes;
mod settings_modifier;
mod voicevox;

use crate::aivoice::AIVOICE;
use crate::icon_manager::ICON_MANAGER;

use anyhow::Result;
use axum::{
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace};
use tracing::{info, Level};

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    ignore_errors = true
)]
struct Cli {
    /// ポート番号。
    #[clap(short, long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let app = Router::new()
        .route("/", get(get_index))
        .route("/version", get(routes::info::get_version))
        .route("/engine_manifest", get(routes::info::get_engine_manifest))
        .route(
            "/supported_devices",
            get(routes::info::get_supported_devices),
        )
        .route("/speakers", get(routes::speakers::get_speakers))
        .route("/speaker_info", get(routes::speakers::get_speaker_info))
        .route(
            "/is_initialized_speaker",
            get(routes::speakers::get_is_initialized_speaker),
        )
        .route("/user_dict", get(routes::user_dict::get_user_dict))
        .route(
            "/import_user_dict",
            post(routes::user_dict::import_user_dict),
        )
        .route("/audio_query", post(routes::audio_query::post_audio_query))
        .route(
            "/accent_phrases",
            post(routes::audio_query::post_accent_phrases),
        )
        .route("/synthesis", post(routes::synthesis::post_synthesis))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::permissive().allow_origin([
            "app://".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
        ]));

    tracing_subscriber::fmt().init();

    AIVOICE.lock().await.setup().await?;

    ICON_MANAGER.lock().await.setup().await?;

    let port = args.port.unwrap_or(50201);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    info!("Speakers:");
    {
        let aivoice = AIVOICE.lock().await;
        for speaker in aivoice.speakers().values() {
            info!(
                "  {} ({}, {})",
                speaker.display_name(),
                speaker.id(),
                speaker.uuid().hyphenated()
            );
        }
    }

    info!("Starting server...");

    info!("Listening on port {}", port);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        })
        .await?;

    info!("Shutting down...");

    AIVOICE.lock().await.shutdown().await?;

    Ok(())
}

async fn get_index() -> impl IntoResponse {
    Redirect::permanent("https://github.com/sevenc-nanashi/aivoice-vox")
}
