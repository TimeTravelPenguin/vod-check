use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, RunArgs};
use dotenvy_macro::dotenv;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;
use twitch_api::helix::videos::get_videos;
use twitch_api::types::{Timestamp, VideoId};
use twitch_api::{TwitchClient, twitch_oauth2::AppAccessToken};

mod cli;

lazy_static! {
    static ref DATE_CUTOFF: Timestamp = Timestamp::from_static("2025-05-01T00:00:00Z");
}

type UserName = String;
type UserID = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DownloadedVideo {
    pub title: String,
    pub published_at: Timestamp,
    pub url: String,
}

impl DownloadedVideo {
    pub fn new(title: String, published_at: Timestamp, url: String) -> Self {
        DownloadedVideo {
            title,
            published_at,
            url,
        }
    }

    pub fn from_twitch_video(video: &twitch_api::helix::videos::Video) -> Self {
        DownloadedVideo {
            title: video.title.clone(),
            published_at: video.published_at.clone(),
            url: video.url.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppOptions {
    pub default_video_date_cutoff: Timestamp,
    pub user_video_date_cutoffs: HashMap<UserName, Timestamp>,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            default_video_date_cutoff: DATE_CUTOFF.clone(),
            user_video_date_cutoffs: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct DownloadHistory {
    pub videos: HashMap<VideoId, Option<DownloadedVideo>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct AppConfig {
    pub app_options: AppOptions,
    pub user_download_history: HashMap<UserID, DownloadHistory>,
}

async fn run_download_test(
    run_args: RunArgs,
    // ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
) -> anyhow::Result<()> {
    let client: TwitchClient<reqwest::Client> = TwitchClient::default();
    let token = AppAccessToken::get_app_access_token(
        &client,
        run_args.twitch_client_id.into(),
        run_args.twitch_client_secret.into(),
        Vec::new(),
    )
    .await
    .context("Failed to get app access token")?;

    let id = &client
        .helix
        .get_user_from_login("smii7y", &token)
        .await
        .context("Failed to get user ID from login")?
        .unwrap()
        .id;

    let vid_req = get_videos::GetVideosRequest::user_id(id);
    let videos = client
        .helix
        .req_get(vid_req, &token)
        .await
        .context("Failed to get videos for user")?
        .data
        .into_iter()
        .filter(|video| DATE_CUTOFF.is_before(&video.published_at))
        .collect::<Vec<_>>();

    let mut app_config = AppConfig::default();

    app_config
        .app_options
        .user_video_date_cutoffs
        .insert("smii7y".to_string(), DATE_CUTOFF.clone());

    let mut download_history = DownloadHistory {
        videos: HashMap::new(),
    };

    for video in videos {
        download_history.videos.insert(
            video.id.clone(),
            Some(DownloadedVideo::from_twitch_video(&video)),
        );
    }

    app_config
        .user_download_history
        .insert(id.to_string(), download_history);

    println!("{:#?}", app_config);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    color_eyre::install().expect("Failed to install color_eyre");

    let cli = Cli::from_env_and_args();

    println!("Parsed CLI arguments:\n{:#?}", cli);

    match cli.command {
        cli::Commands::Tui => {
            // Placeholder for TUI implementation
            println!("TUI mode is not implemented yet.");
            Ok(())
        }
        cli::Commands::Run { args } => run_download_test(args).await,
    }?;

    Ok(())
}
