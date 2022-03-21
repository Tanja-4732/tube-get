use anyhow::{anyhow, Result};
use reqwest::{cookie::Jar, Client, Url};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    constants,
    types::{
        episodes::{EpisodesData, TrackType},
        oof,
    },
};

pub async fn get_episodes(
    client: &Client,
    offset: u64,
    limit: u64,
    uuid: Uuid,
    verbosity: u64,
) -> Result<EpisodesData> {
    println!("Fetch JSON from the API...");

    let url = format!(
        "https://tube.tugraz.at/search/episode.json?limit={limit}&offset={offset}&sid={uuid}",
        uuid = uuid.to_string()
    );

    if verbosity > 0 {
        println!("Using URL: {}", &url);
    }

    let text = client.get(url).send().await?.text().await?;

    let parsed = serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&text));
    parsed.map_err(|e| {
        if verbosity >= 4 {
            println!("---Begin of full text dump---\n{text}\n---End of full text dump---");
        }

        if verbosity >= 1 {
            println!("Full error:\n{}", e.path());
        }

        if serde_json::from_str::<oof::Root>(&text).is_ok() {
            anyhow!("Is your login token (still) valid? Please provide a recent JSESSIONID cookie.")
        } else {
            anyhow!("Couldn't parse JSON from the API. Unknown error: {}", e)
        }
    })
}

pub fn make_client(token: &str) -> Result<Client> {
    let url = Url::parse(constants::BASE_URL)?;

    let jar = Jar::default();
    jar.add_cookie_str(&format!("JSESSIONID={}", token), &url);

    Ok(Client::builder().cookie_provider(jar.into()).build()?)
}

pub fn extract_course_data(data: &EpisodesData) -> Result<Course> {
    println!("Extracting data...");

    let first = data.search_results.result.get(0).unwrap();

    // TODO implement progress bar or remove it
    // let pb = ProgressBar::new(data.search_results.result.len().try_into().unwrap());

    let mut videos = Vec::new();

    for result in &data.search_results.result {
        let tracks = result.mediapackage.media.track.iter().filter(|track| {
            track.mimetype == constants::MP4_MIME
                && track
                    .tags
                    .tag
                    .contains(&constants::HIGH_QUALITY.to_string())
        });

        for track in tracks {
            videos.push(Video {
                url: track.url.to_owned(), // TODO change this back to a borrow
                title: result.mediapackage.title.to_owned(), // TODO change this back to a borrow
                id: result.id.to_owned(),  // TODO change this back to a borrow
                video_type: track.type_field,
            });
        }
    }

    let course = Course {
        title: first.mediapackage.seriestitle.to_owned(), // TODO change this back to a borrow
        id: first.mediapackage.series.to_owned(),         // TODO change this back to a borrow
        videos,
    };

    Ok(course)
}

// #[derive(Debug, Serialize, Clone)]
// pub struct Video<'a> {
//     pub url: &'a str,
//     pub title: &'a str,
//     pub id: &'a str,
//     pub video_type: TrackType,
// }

// #[derive(Debug, Serialize, Clone)]
// pub struct Course<'a> {
//     pub title: &'a str,
//     pub id: &'a str,
//     pub videos: Vec<Video<'a>>,
// }

#[derive(Debug, Serialize, Clone)]
pub struct Video {
    pub url: String,
    pub title: String,
    pub id: String,
    pub video_type: TrackType,
}

#[derive(Debug, Serialize, Clone)]
pub struct Course {
    pub title: String,
    pub id: String,
    pub videos: Vec<Video>,
}
