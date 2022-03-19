use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodesData {
    #[serde(rename = "search-results")]
    pub search_results: SearchResults,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
    pub search_time: i64,
    pub query: String,
    #[serde(deserialize_with = "one_or_more_result")]
    pub result: Vec<Result>,
}

fn one_or_more_result<'de, D>(deserializer: D) -> std::result::Result<Vec<Result>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    // TODO care about the following warning: large size difference between variants
    enum OneOrMore {
        One(Result),
        More(Vec<Result>),
    }

    // Attempts de-serialization
    let one_or_more = match serde_path_to_error::deserialize(deserializer) {
        Ok(one_or_more_deserialized) => one_or_more_deserialized,
        Err(err) => panic!("Full error path: {}", err.path()),
    };

    // If we got one object instead of a vector, wrap it in a vector
    Ok(match one_or_more {
        OneOrMore::One(the_one) => vec![the_one],
        OneOrMore::More(the_more) => the_more,
    })
}

fn one_or_more_string<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMore {
        One(String),
        More(Vec<String>),
    }

    Ok(match OneOrMore::deserialize(deserializer)? {
        OneOrMore::One(the_one) => vec![the_one],
        OneOrMore::More(the_more) => the_more,
    })
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub id: String,
    pub org: String,
    pub mediapackage: Mediapackage,
    // TODO uncomment the lines below
    // pub dc_extent: i64,
    // pub dc_title: String,
    // pub dc_creator: Option<String>,
    // pub dc_publisher: Option<String>,
    // pub dc_created: String,
    // pub dc_spatial: String,
    // pub dc_is_part_of: String,
    // pub oc_mediapackage: String,
    // pub media_type: String,
    // pub keywords: Keywords,
    // pub modified: String,
    // pub score: f64,
    // pub segments: Option<Segments>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mediapackage {
    pub duration: i64,
    pub id: String,
    pub start: String,
    pub title: String,
    pub series: String,
    pub seriestitle: String,
    pub media: Media,
    pub metadata: Metadata,
    pub attachments: Attachments,
    pub publications: String,
    // I got a response with [0, "text"] once instead of just "text"
    // TODO handle an array with both strings and numbers for some reason
    // pub creators: Option<Creators>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub track: Vec<Track>,
}

/// A candidate for downloading
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: TrackType,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub mimetype: String,
    /// Contains info on the videos quality
    pub tags: Tags,
    pub url: String,
    pub checksum: Option<Checksum>,
    pub duration: i64,
    // TODO uncomment the lines below
    // /// Should be `Some` for the things we care about
    // pub audio: Option<Audio>,
    // /// Should be `Some` for the things we care about
    // pub video: Option<Video>,
    // pub live: bool,
    /// Should be `None` for the things we care about
    pub transport: Option<String>,
    // pub size: Option<i64>,
    // pub master: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub enum TrackType {
    #[serde(rename = "presenter/delivery")]
    Presenter,
    #[serde(rename = "presentation/delivery")]
    Presentation,
    #[serde(rename = "raw/delivery")]
    Raw,
}

impl Default for TrackType {
    fn default() -> Self {
        TrackType::Presentation
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    #[serde(deserialize_with = "one_or_more_string")]
    pub tag: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "$")]
    pub field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    pub id: String,
    pub device: String,
    pub encoder: Encoder,
    pub framecount: i64,
    pub channels: i64,
    pub samplingrate: i64,
    pub bitrate: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encoder {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub device: String,
    pub encoder: Encoder,
    pub framecount: i64,
    pub bitrate: f64,
    pub framerate: f64,
    pub resolution: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub catalog: Vec<Catalog>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub mimetype: String,
    pub tags: Tags,
    pub url: String,
    pub checksum: Option<Checksum2>,
    #[serde(rename = "ref")]
    pub ref_field: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum2 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "$")]
    pub field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachments {
    pub attachment: Vec<Attachment>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: Option<String>,
    pub mimetype: String,
    pub tags: Tags,
    pub url: String,
    pub size: Option<i64>,
    pub additional_properties: Option<AdditionalProperties>,
    pub checksum: Option<Checksum3>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum3 {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "$")]
    pub field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalProperties {
    pub property: Vec<Property>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub key: String,
    #[serde(rename = "$")]
    pub field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creators {
    pub creator: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Keywords {
    #[serde(deserialize_with = "one_or_more_string")]
    pub keywords: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segments {
    pub segment: Vec<Segment>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub index: i64,
    pub time: i64,
    pub duration: i64,
    pub relevance: i64,
    pub hit: bool,
    pub text: String,
    pub previews: Previews,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Previews {
    pub preview: Preview,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    #[serde(rename = "ref")]
    pub ref_field: String,
    #[serde(rename = "$")]
    pub field: String,
}
