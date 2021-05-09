use anyhow::Result;
use rayon::prelude::*;
use reqwest::{cookie::Jar, Client, Url};
use serde::Serialize;
use uuid::Uuid;

use crate::{cli::CliOptions, constants, types::episodes::EpisodesData};

pub async fn get_episodes(
    client: &Client,
    offset: u64,
    limit: u64,
    uuid: Uuid,
) -> Result<EpisodesData> {
    let url = format!(
        "https://tube.tugraz.at/search/episode.json?limit={limit}&offset={offset}&sid={uuid}",
        limit = limit,
        offset = offset,
        uuid = uuid.to_string()
    );

    Ok(client.get(url).send().await?.json::<EpisodesData>().await?)
}

pub fn make_client() -> Result<Client> {
    let url = Url::parse(&constants::BASE_URL)?;

    let jar = Jar::default();
    jar.add_cookie_str(
        "JSESSIONID=node0fy5phk65e0ni1m40w0n3xgka01174409.node0	",
        &url,
    );

    Ok(Client::builder().cookie_provider(jar.into()).build()?)
}

pub fn extract_course_data(data: &EpisodesData) -> Result<Course> {
    let videos = data
        .search_results
        .result
        .par_iter()
        .map(|result| Video {
            url: &result
                .mediapackage
                .media
                .track
                .iter()
                .find(|track| {
                    track.mimetype == constants::MP4_MIME
                        && track
                            .tags
                            .tag
                            .contains(&constants::HIGH_QUALITY.to_string())
                })
                .unwrap()
                .url,
            title: &result.mediapackage.title,
            id: &result.id,
        })
        .collect();

    let first = data.search_results.result.get(0).unwrap();

    let course = Course {
        title: &first.mediapackage.seriestitle,
        id: &first.mediapackage.series,
        videos,
    };

    Ok(course)
}

#[derive(Debug, Serialize)]
pub struct Video<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub id: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Course<'a> {
    pub title: &'a str,
    pub id: &'a str,
    pub videos: Vec<Video<'a>>,
}
