use anyhow::{anyhow, Result};
use ocapi::{
    client::{search::SearchParams, AuthToken, OcApi},
    core::{
        api::Api,
        search::{self, EpisodesData},
    },
};
use reqwest::{cookie::Jar, Url};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    constants,
    types::{episodes::TrackType, oof},
};

pub async fn get_episodes(
    api: &OcApi,
    offset: u64,
    limit: u64,
    series_uuid: Uuid,
    verbosity: u64,
) -> Result<EpisodesData> {
    let limit = limit.to_string().into();
    let offset = offset.to_string().into();
    let series_uuid = series_uuid.to_string().into();

    println!("Fetch JSON from the API...");

    let params = SearchParams {
        limit,
        offset,
        series_uuid,
    };

    // TODO remove this, move to the OcApi crate
    if verbosity > 0 {
        println!("Using URL: https://tube.tugraz.at/search/episode.json?limit={limit}&offset={offset}&sid={uuid}",
        limit = params.limit.as_ref().unwrap(), offset = params.offset.as_ref().unwrap(), uuid = params.series_uuid.as_ref().unwrap());
    }

    let episodes = api.search_episode(&params).await;

    episodes.map_err(|e| {
        // if verbosity >= 4 {
        //     println!("---Begin of full text dump---\n{text}\n---End of full text dump---");
        // }

        // if verbosity >= 1 {
        //     println!("Full error:\n{}", e.path());
        // }

        // if serde_json::from_str::<oof::Root>(&text).is_ok() {
        //     anyhow!("Is your login token (still) valid? Please provide a recent JSESSIONID cookie.")
        // } else {
        //     anyhow!("Couldn't parse JSON from the API. Unknown error: {}", e)
        // }

        // TODO
        anyhow!("Couldn't parse JSON from the API. Unknown error: {}", e)
    })
}

pub fn make_client(token: &str) -> OcApi {
    OcApi::new(
        constants::BASE_URL.to_string(),
        AuthToken::new("JSESSIONID".to_string(), token.to_string()),
    )
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

mod test {
    #[test]
    fn test_get_episodes() {
        use super::*;
        use std::fs;
        let text = fs::read_to_string("private/episodes_test.json").unwrap();

        let parsed: anyhow::Result<EpisodesData> =
            serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&text))
                .map_err(anyhow::Error::from);

        let data = parsed.unwrap();
    }
}
