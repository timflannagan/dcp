mod error;

use clap::{App, Arg};
use std::error::Error;
use docker_api::{Docker};
use docker_api::api::{PullOpts, ContainerCreateOpts};
use std::path::PathBuf;
use tar::Archive;
use futures_util::{StreamExt, TryStreamExt};
use crate::error::DCPError;

pub type DCPResult<T> = Result<T, Box<dyn Error>>;

const DOCKER_SOCKET: &str = "unix:///var/run/docker.sock";
pub const VERSION: &str = "0.1.0";


#[derive(Debug)]
pub struct Config {
    // List of images
    image: String,
    // Where the download files should be saved on the filesystem. Default "."
    download_path: String,
    // Where the content (files) are in the container filesystem. Default "/"
    content_path: String,
    // Option to write to stdout instead of the local filesystem.
    write_to_stdout: bool,
}

pub fn get_args() -> DCPResult<Config> {
    let matches = App::new("dcp")
        .version(VERSION)
        .author("exdx")
        .about("docker cp made easy")
        .arg(
            Arg::with_name("image")
                .value_name("IMAGE")
                .help("Container image to extract content from")
                .required(true)
        )
        .arg(
            Arg::with_name("download_path")
                .value_name("DOWNLOAD_PATH")
                .help("Where the image contents should be saved on the filesystem")
                .default_value(".")
                .short("d")
                .long("download_path")
        )
        .arg(
            Arg::with_name("content_path")
                .value_name("CONTENT_PATH")
                .help("Where in the container filesystem the content to extract is")
                .short("p")
                .default_value("/")
                .long("content_path")
        )
        .arg(
            Arg::with_name("write_to_stdout")
                .value_name("WRITE_TO_STDOUT")
                .help("Whether to write to stdout instead of the filesystem")
                .takes_value(false)
                .short("w")
                .long("write_to_stdout")
        ).get_matches();

    let image = matches.value_of("image").unwrap().to_string();
    let download_path = matches.value_of("download_path").unwrap().to_string();
    let content_path = matches.value_of("content_path").unwrap().to_string();
    let write_to_stdout = matches.is_present("write_to_stdout");

    if write_to_stdout {
        return Err(Box::new(DCPError::new("error: writing to stdout is not currently implemented")));
    }

    Ok(Config {
        image,
        download_path,
        content_path,
        write_to_stdout,
    })
}

pub struct Image{
    image: String
}

impl Image{
    // Returns a Result with an Image struct and an Error (if one occured)
    fn split_repo_and_tag(&self) -> (String, String) {
        let image_split: Vec<&str> = self.image.split(":").collect();

        let mut repo = String::new();
        if let Some(i) = image_split.get(0) {
            repo = i.to_string();
        }

        let mut tag = String::new();
        if let Some(i) = image_split.get(1) {
            tag = i.to_string();
        }

        return (repo, tag);
    }
}


/// Run runs a sequence of events with the provided image
/// 1. Pull down the image
/// 2. Create a container, receiving the container id as a response
/// 3. Copy the container content to the specified directory
pub async fn run(config: Config) -> DCPResult<()> {
    let docker = Docker::new(DOCKER_SOCKET)?;

    let image = Image{image: config.image.clone()};
    let (repo, tag) = image.split_repo_and_tag();

    let pull_opts = PullOpts::builder().image(repo).tag(tag).build();
    let images = docker.images();
    let mut stream = images.pull(&pull_opts);

    while let Some(pull_result) = stream.next().await {
        match pull_result {
            Ok(output) => {
                println!("{:?}", output);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    let create_opts = ContainerCreateOpts::builder(config.image.clone()).build();
    let container = docker.containers().create(&create_opts).await?;
    let id = container.id();
    println!("{:?}", id);

    let mut content_path = PathBuf::new();
    content_path.push(&config.content_path);

    let mut download_path = PathBuf::new();
    download_path.push(&config.download_path);

    let bytes = docker
        .containers()
        .get(&*id)
        .copy_from(&content_path)
        .try_concat()
        .await?;

    let mut archive = Archive::new(&bytes[..]);
    if config.write_to_stdout {
        unimplemented!()
    } else {
        archive.unpack(&download_path)?;
    }

    Ok(())
}
