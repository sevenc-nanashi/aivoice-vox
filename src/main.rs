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
use crate::routes::audio_query::OPEN_JTALK;
use crate::routes::user_dict::USER_DICT;

use anyhow::Result;
use axum::{
    response::{IntoResponse, Redirect},
    routing::{delete, get, post, put},
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

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(cfg!(debug_assertions))
        .init();

    AIVOICE.lock().await.setup().await?;

    let result = main_impl(args).await;

    info!("Shutting down...");

    AIVOICE.lock().await.shutdown().await?;

    result?;

    Ok(())
}

async fn main_impl(args: Cli) -> Result<()> {
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
        .route(
            "/user_dict_word",
            post(routes::user_dict::post_user_dict_word),
        )
        .route(
            "/user_dict_word/:word_uuid",
            delete(routes::user_dict::delete_user_dict_word),
        )
        .route(
            "/user_dict_word/:word_uuid",
            put(routes::user_dict::put_user_dict_word),
        )
        .route("/audio_query", post(routes::audio_query::post_audio_query))
        .route(
            "/accent_phrases",
            post(routes::audio_query::post_accent_phrases),
        )
        .route("/synthesis", post(routes::synthesis::post_synthesis))
        .layer(CorsLayer::permissive())
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

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

    {
        let open_jtalk = OPEN_JTALK.lock().await;
        let user_dict = USER_DICT.lock().await;
        info!("Loading OpenJTalk dictionary...");
        open_jtalk.use_user_dict(&user_dict)?;
    }
    info!("Listening on port {}", port);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
        })
        .await?;

    Ok(())
}

async fn get_index() -> impl IntoResponse {
    Redirect::permanent("https://github.com/sevenc-nanashi/aivoice-vox")
}
