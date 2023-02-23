use chrono;
use serde_json::Value;
use youtube_dl::YoutubeDl;

// TODO: add like scraping; needs OAuth2.0

fn get_saved_video_list(file_exists: bool, filepath: String) -> Vec<String> {
    if file_exists {
        let prev_text = std::fs::read_to_string(&filepath).unwrap();
        let mut prev_update: Vec<String> = prev_text.split(",").map(String::from).collect();
        prev_update.dedup();
        prev_update
    } else {
        Vec::new()
    }
}

fn get_video_list(playlist_id: String, api_key: String) -> Vec<String> {
    let agent = ureq::agent();
    let playlist_url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&key={}&maxResults=1",
        playlist_id,
        api_key
    );

    let response = agent
        .get(&playlist_url)
        .call()
        .expect("Invalid http response");
    let response_text = response.into_string().expect("Failed to get response text");

    let mut final_result_vec: Vec<String> = Vec::new();
    let mut latest_updates =
        serde_json::from_str::<Value>(&response_text.trim()).expect("Failed to parse JSON");

    let playlist_length = latest_updates["items"].as_array().unwrap().len();

    for index in 0..playlist_length {
        final_result_vec
            .push(latest_updates["items"][index]["snippet"]["resourceId"]["videoId"].to_string());
    }

    final_result_vec.dedup();

    while !latest_updates["nextPageToken"].is_null() {
        let playlist_url_paginated = playlist_url.clone()
            + "&pageToken="
            + &latest_updates["nextPageToken"]
                .to_string()
                .replace("\"", "");
        let response_paginated = agent
            .get(&playlist_url_paginated)
            .call()
            .expect("Invalid http response");
        let response_text_url = response_paginated
            .into_string()
            .expect("Failed to get response text");

        latest_updates =
            serde_json::from_str::<Value>(&response_text_url.trim()).expect("Failed to parse JSON");

        let playlist_elements = latest_updates["items"].as_array().unwrap().len();

        for index in 0..playlist_elements {
            final_result_vec.push(
                latest_updates["items"][index]["snippet"]["resourceId"]["videoId"].to_string(),
            );
        }
        final_result_vec.dedup();
    }

    final_result_vec
}

pub fn run_ytdl() {

    let api_key = std::env::var("YT_API_KEY").expect("An API key is required for this program to run.").to_owned();
    let playlist_id = std::env::var("PLAYLIST_ID").expect("A playlist to be tracked is required.").to_owned();
    let folder_path = std::env::var("FILEPATH").unwrap_or(".".to_string());

    let filepath = format!("{}/{}", folder_path, "playlist.txt");
    let file_exists = std::path::Path::new(&filepath).exists();
    let video_ids: Vec<String> = get_video_list(playlist_id, api_key);
    let prev_update = get_saved_video_list(file_exists, filepath.to_string());

    let differences: Vec<String> = video_ids
        .clone()
        .into_iter()
        .partition(|x| prev_update.contains(x))
        .1;

    // write playlist back to file
    std::fs::write(filepath, &video_ids.join(",")).expect("Unable to write file");

    if differences.is_empty() {
        // do nothing
        println!("No changes detected - {}", chrono::Utc::now());
    } else {
        // download differences
        println!("Changes detected - {}", chrono::Utc::now());
        println!("found {} video(s)", differences.len());
        for video_id in differences {
            let video_link = format!("https://www.youtube.com/watch?v={}", video_id.replace("\"", ""));
            let query_run_result = YoutubeDl::new(video_link)
                .socket_timeout("15")
                .download(true)
                .format("m4a")
                .output_directory(format!("{}/downloads/", folder_path))
                .run();

            match query_run_result {
                Ok(result) => {
                    let title = result.into_single_video().unwrap().title;
                    println!("{}", title);
                }
                Err(e) => {
                    println!("{:?}", e)
                }
            }
        }
    }
}

//23 seconds to get details of 18 videos. Slow but an alternative to using API key
// let md = chrono::Utc::now();
// println!("{}",YoutubeDl::new("https://www.youtube.com/playlist?list=PLw9di5JwI6p-HUP0yPUxciaEjrsFb2kR2").youtube_dl_path("./yt-dlp.exe").socket_timeout("15").run().unwrap().into_playlist().unwrap().entries.unwrap().to_vec().len());
// println!("Total time taken: {}",chrono::Utc::now()-md);
