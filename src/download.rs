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

use std::fs;
use std::io;

use crate::{cli::CliOptions, extractor::Course};

pub async fn download_course(cli_options: &CliOptions<'_>, course: &Course<'_>) -> Result<()> {
    println!("Downloading course {}", course.title);

    let multibar = MultiProgress::new();

    let main_pb = multibar.add(ProgressBar::new(course.videos.len() as u64));

    main_pb.set_style(ProgressStyle::default_bar().template("{msg} {bar:10} {pos}/{len}"));
    main_pb.set_message("total  ");

    // Make the main progress bar render immediately rather than waiting for the first task to finish.
    main_pb.tick();

    let folder_path = fs::canonicalize(cli_options.destination.clone())?.join(course.title);

    fs::create_dir_all(&folder_path)?;

    let mut count = cli_options.skip_count.unwrap_or(0) as usize;

    for video in course.videos.iter()
    /* .skip(count) */ // We already skip server-side
    {
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(
                folder_path
                    .clone()
                    .join(video.title.to_string().replace("/", "_") + ".mp4"),
            )?;

        let client = Client::new();

        let res = client.head(video.url).send().await?;

        let download_size = res
            .headers() // Gives us the HeaderMap
            .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
            .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
            .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
            .unwrap_or(0); // Fallback to 0

        let request = client.get(video.url);

        let progress_bar = multibar.add(ProgressBar::new(download_size));

        // Set Style to the ProgressBar
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} - {msg}")
                .progress_chars("#>-"),
        );

        progress_bar.set_message(video.title.to_string());

        let mut download = request.send().await?;
        let mut writer = io::BufWriter::new(file);

        while let Some(chunk) = download.chunk().await? {
            progress_bar.inc(chunk.len() as u64); // Increase ProgressBar by chunk size
            progress_bar.tick();
            writer.write(&chunk)?; // Write chunk to output file
        }

        progress_bar.finish();

        writer.flush()?;

        count += 1;
        println!(
            "  Finished {:3}/{:3}: {}",
            count,
            course.videos.len() + cli_options.skip_count.unwrap_or(0) as usize,
            video.title
        );

        main_pb.inc(1);
    }

    main_pb.finish();

    println!("Download complete.");

    Ok(())
}
