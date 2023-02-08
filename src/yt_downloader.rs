use chrono;
use serde_json::Value;
use youtube_dl::YoutubeDl;

pub fn run_ytdl() {
    let user_playlist_id = "{PlaylistID}&key={YOUR_API_KEY}";

    let path = match std::env::var("FILEPATH") {
        Ok(path) => path,
        Err(_error) => "./".to_owned(),
    };

    let playlist_id = match std::env::var("ID") {
        Ok(playlist_id) => playlist_id,
        Err(_error) => user_playlist_id.to_owned(),
    };

    let filename = "playlist.txt";
    let filepath = &(path.to_owned() + filename);
    let agent = ureq::agent();
    let playlist_url =
        "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId=".to_owned()
            + &playlist_id
            + "&maxResults=1";
    let response = agent
        .get(&playlist_url)
        .call()
        .expect("Invalid http response");
    let response_text = response.into_string().expect("Failed to get response text");
    let file_exists = std::path::Path::new(filepath).exists();

    //Fundamentally wrong way of doing things. Change to push to vec and re-get next page

    let mut final_result_vec: Vec<String> = Vec::new();
    let mut latest_updates =
        serde_json::from_str::<Value>(&response_text.trim()).expect("Failed to parse JSON");

    let playlist_elements = latest_updates["items"].as_array().unwrap().len();

    for index in 0..playlist_elements {
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
    let mut changed_vec: Vec<String> = final_result_vec.clone();
    if file_exists {
        let prev_text = std::fs::read_to_string(&filepath).unwrap();
        let mut prev_update: Vec<&str> = prev_text.split(",").collect::<Vec<&str>>();

        prev_update.dedup();

        changed_vec = final_result_vec
            .clone()
            .into_iter()
            .partition(|x| prev_update.contains(&x.as_str()))
            .1;

        std::fs::write(filepath, &final_result_vec.join(",")).expect("Unable to write file");
    } else {
        std::fs::write(filepath, &final_result_vec.join(",")).expect("Unable to write file");
    }

    let flag = if file_exists {
        if changed_vec.is_empty() {
            println!("No changes detected - {}", chrono::Utc::now());
            false
        } else {
            println!("Changes detected - {}", chrono::Utc::now());
            true
        }
    } else {
        true
    };

    if flag {
        for index in changed_vec {
            let video_link =
                "https://www.youtube.com/watch?v=".to_owned() + &index.replace("\"", "");
            let output = YoutubeDl::new(video_link)
                .youtube_dl_path(path.clone() + "yt-dlp.exe")
                .socket_timeout("15")
                .download(true)
                .output_directory(path.clone() + "downloads/")
                .run()
                .unwrap()
                .into_single_video()
                .unwrap()
                .title;

            println!("{}", output);
        }
    }
}

//23 seconds to get details of 18 videos. Slow but an alternative to using API key
// let md = chrono::Utc::now();
// println!("{}",YoutubeDl::new("https://www.youtube.com/playlist?list=PLw9di5JwI6p-HUP0yPUxciaEjrsFb2kR2").youtube_dl_path("./yt-dlp.exe").socket_timeout("15").run().unwrap().into_playlist().unwrap().entries.unwrap().to_vec().len());
// println!("Total time taken: {}",chrono::Utc::now()-md);
