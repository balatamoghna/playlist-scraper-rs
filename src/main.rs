mod yt_downloader;

use actix_files::NamedFile;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};

/*
fn main() {
    println!(
        "A simple scraping bot to check for updates in Youtube Playlist.\nThank you for using!"
    );
    //yt_downloader::run_ytdl();
}
*/
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("A simple scraping REST API to save a Youtube playlist.\nThank you for using!");

    HttpServer::new(|| {
        App::new()
            .service(test_file)
            .service(list_files)
            .service(len)
            .service(download)
            .service(forcedl)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/test")]
async fn test_file() -> Result<NamedFile> {
    Ok(NamedFile::open(
        "./downloads/".to_owned()
            + &std::fs::read_dir("./downloads")
                .unwrap()
                .nth(0)
                .unwrap()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap(),
    )?)
}

#[get("/list")]
async fn list_files() -> impl Responder {
    let mut files: Vec<String> = vec![];
    for file in std::fs::read_dir("./downloads").unwrap() {
        files.push(file.unwrap().file_name().into_string().unwrap());
    }

    HttpResponse::Ok().body(files.join("\n"))
}

#[get("/len")]
async fn len() -> impl Responder {
    HttpResponse::Ok().body(
        std::fs::read_dir("./downloads")
            .unwrap()
            .count()
            .to_string(),
    )
}

#[get("/download/{index}")]
async fn download(req: HttpRequest) -> Result<NamedFile> {
    let index: u8 = req.match_info().get("index").unwrap().parse().unwrap();
    if index - 1 > 0 && (index as usize) - 1 < std::fs::read_dir("./downloads").unwrap().count() {
        Ok(NamedFile::open(
            "./downloads/".to_owned()
                + &std::fs::read_dir("./downloads")
                    .unwrap()
                    .nth(index as usize - 1)
                    .unwrap()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap(),
        )?)
    } else {
        Ok(NamedFile::open(
            "./downloads/".to_owned()
                + &std::fs::read_dir("./downloads")
                    .unwrap()
                    .nth(0)
                    .unwrap()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap(),
        )?)
    }
}

#[get("/forcedl")]
async fn forcedl() -> impl Responder {
    yt_downloader::run_ytdl();
    HttpResponse::Ok().body("Running Youtube Downloader")
}
