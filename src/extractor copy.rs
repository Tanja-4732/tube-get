use anyhow::Result;
use hyper::{body::HttpBody as _, Client, Uri};
use hyper_tls::HttpsConnector;
use tokio::io::{stdout, AsyncWriteExt as _};
use uuid::Uuid;

use crate::cli::CliOptions;

pub async fn test(cli_options: &CliOptions) -> Result<()> {
    // Make a HTTPS client
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let mut res = client.get(cli_options.uri.clone()).await?;

    println!("Response: {}", res.status());

    while let Some(chunk) = res.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}

pub async fn get_episodes(cli_options: &CliOptions, uuid: Uuid) -> Result<()> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let url = format!(
        "https://tube.tugraz.at/search/episode.json?limit={limit}&offset={offset}&sid={uuid}",
        limit = cli_options.limit_count.unwrap_or(99999),
        offset = cli_options.skip_count.unwrap_or(0),
        uuid = uuid.to_string()
    );

    let uri = url.parse()?;

    let mut res = client.get(uri).await?;

    Ok(())
}
