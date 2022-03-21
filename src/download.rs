use anyhow::Result;
use indicatif::{HumanBytes, HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use io::Write;
use reqwest::Client;

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

    main_pb.set_style(
        ProgressStyle::default_bar().template(
            &(String::new() + "[{elapsed_precise}] {wide_bar} Total progress: {pos}/{len}"),
        ),
    );

    main_pb.println(format!("Downloading course {}", course.title));

    // Make the main progress bar render immediately rather than waiting for the first task to finish.
    main_pb.tick();

    let folder_path = PathBuf::from(cli_options.destination.to_owned()).join(course.title);

    // TODO Change this to a question mark operator somehow
    fs::create_dir_all(&folder_path).expect("Could not create directory");

    let mut count = cli_options.skip_count.unwrap_or(0) as usize;

    async move {
        for video in course.videos.iter() {
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .open(folder_path.clone().join(create_video_file_name(video)))?;

            let client = Client::new();

            let request = client.get(&video.url);

            let mut response = request.send().await?;
            let mut writer = io::BufWriter::new(file);

            let content_length = response.content_length().unwrap_or(0);

            if cli_options.verbosity >= 1 {
                main_pb.println(format!("Downloading URL: {}", &video.url));
            }

            if cli_options.verbosity >= 2 {
                main_pb.println(format!("Status code: {:#?}", &response.status()));
                main_pb.println(format!("Headers:\n{:#?}", &response.headers()));
            }

            let progress_bar = multi_bar.add(
                ProgressBar::new(content_length)
                    .with_style(
                        ProgressStyle::default_bar()
                            .template(
                                "[{elapsed_precise}] {wide_bar:.cyan} {bytes}/{total_bytes} ({percent}%) - ETA {eta_precise} with {binary_bytes_per_sec} - {msg}"
                            )
                            .progress_chars("█▉▊▋▌▍▎▏  "),
                    )
                    .with_message(video.title.to_owned()),
            );

            while let Some(chunk) = response.chunk().await? {
                progress_bar.inc(chunk.len() as u64); // Increase ProgressBar by chunk size
                main_pb.tick();
                writer.write_all(&chunk)?; // Write chunk to output file
            }

            let elapsed = HumanDuration(progress_bar.elapsed());
            let downloaded = HumanBytes(progress_bar.position());

            progress_bar.finish_and_clear();

            writer.flush()?;

            count += 1;
            main_pb.println(format!(
                "  Finished {:2}/{:2}, {downloaded} in {elapsed}: {}",
                count,
                course.videos.len() + cli_options.skip_count.unwrap_or(0) as usize,
                video.title
            ));

            main_pb.inc(1);
        }

        main_pb.finish();

        println!("Download complete.");
        Ok(())
    }
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
