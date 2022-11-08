// The name of the application
pub const NAME: &str = "tube-get";

/// The main author of the application
pub const AUTHOR: &str = "Tanja <git@tanja.pw>";

/// The semantic-version string of the application
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Describes the application (i.e. its use cases) in a short phrase
pub const ABOUT: &str =
    "A Rust tool for recursively crawling & downloading videos from TU Graz TUbe";

/// The licence notice (AGPL 3) of the application
pub const LICENSE: &str = concat![
    "Copyright 2021 Tanja; All rights reserved.\n",
    "Licensed under the AGPL 3.0 <https://www.gnu.org/licenses/agpl-3.0.en.html>"
];

/// The base URL
pub const BASE_URL: &str = "https://tube.tugraz.at";

/// The mp4 mimetype
pub const MP4_MIME: &str = "video/mp4";

pub const HIGH_QUALITY: &str = "high";
