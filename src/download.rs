use anyhow::Result;
use indicatif::{
    MultiProgress,
    ProgressBar,
    ProgressStyle,
    // ParallelProgressIterator, ProgressIterator,
};
use io::Write;
// use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reqwest::{header, Client};

use std::fmt::Display;
use std::fs;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use crate::types::episodes::TrackType;
use crate::{cli::CliOptions, extractor::Course};

pub fn download_course<'a>(
    cli_options: CliOptions,
    course: Course,
    multi_bar: Arc<MultiProgress>,
) -> impl Future<Output = Result<()>> + 'a {
    let main_pb = multi_bar.add(ProgressBar::new(course.videos.len() as u64));

    main_pb.set_style(ProgressStyle::default_bar().template("{msg} {bar:10} {pos}/{len}"));
    main_pb.set_message("total  ");

    main_pb.println(format!("Downloading course {}", course.title));

    // Make the main progress bar render immediately rather than waiting for the first task to finish.
    main_pb.tick();

    let folder_path = PathBuf::from(cli_options.destination.to_owned()).join(course.title);

    // TODO Change this to a question mark operator somehow
    fs::create_dir_all(&folder_path).expect("Could not create directory");

    let mut count = cli_options.skip_count.unwrap_or(0) as usize;

    let task = async move {
        for video in course.videos.iter()
        /* .skip(count) */ // We already skip server-side
        {
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .open(folder_path.clone().join(create_video_file_name(video)))?;

            let client = Client::new();

            let res = client.head(&video.url).send().await?;

            let download_size = res
                .headers() // Gives us the HeaderMap
                .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
                .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                .unwrap_or(0); // Fallback to 0

            if cli_options.verbosity >= 1 {
                main_pb.println(format!("Downloading URL: {}", &video.url));
            }

            if cli_options.verbosity >= 2 {
                main_pb.println(format!("Headers:\n{:#?}", &res.headers()));
            }

            let request = client.get(&video.url);

            let progress_bar = multi_bar.add(
                ProgressBar::new(download_size)
                    .with_style(
                        ProgressStyle::default_bar()
                            // .template("{bar:100.cyan} {pos:>4}/{len:4}")
                            .template("{bar:100.cyan} {bytes}/{total_bytes} - {msg}")
                            .progress_chars("█▉▊▋▌▍▎▏  "),
                    )
                    .with_message(video.title.to_owned()),
            );

            let mut download = request.send().await?;
            let mut writer = io::BufWriter::new(file);

            while let Some(chunk) = download.chunk().await? {
                progress_bar.inc(chunk.len() as u64); // Increase ProgressBar by chunk size
                writer.write_all(&chunk)?; // Write chunk to output file
            }

            progress_bar.finish();

            writer.flush()?;

            count += 1;
            main_pb.println(format!(
                "  Finished {:3}/{:3}: {}",
                count,
                course.videos.len() + cli_options.skip_count.unwrap_or(0) as usize,
                video.title
            ));

            main_pb.inc(1);
        }

        main_pb.finish();

        println!("Download complete.");
        Ok(())
    };

    task
}

fn create_video_file_name(video: &crate::extractor::Video) -> String {
    video.title.to_string().replace("/", "_") + "_" + &video.video_type.to_string() + ".mp4"
}

impl Display for TrackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lower = match self {
            TrackType::Presenter => "presenter",
            TrackType::Presentation => "presentation",
            TrackType::Raw => "raw",
        };

        f.write_str(lower)
    }
}
