use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::{
    config::{Builder, Credentials, Region},
    Client,
};
use clap::{ArgAction::SetTrue, Parser, ValueEnum};
use http::{uri::PathAndQuery, Uri};
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Parser)]
struct Cli {
    endpoint_url: Url,
    bucket: String,
    mode: Mode,
    #[arg(long, env)]
    access_key_id: Option<String>,
    #[arg(long, env)]
    secret_access_key: Option<String>,
    #[arg(long, env, action = SetTrue)]
    force_path_style: bool,
    #[arg(long, env)]
    region: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    Standard,
    Modified,
}

fn setup_client(
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    endpoint_url: Url,
    force_path_style: bool,
    region: Option<String>,
) -> Client {
    let credentials = Credentials::new(
        access_key_id.unwrap_or_default(),
        secret_access_key.unwrap_or_default(),
        None,
        None,
        "minimal-repro",
    );
    let credentials_provider = SharedCredentialsProvider::new(credentials);
    let config = Builder::new()
        .credentials_provider(credentials_provider)
        .endpoint_url(endpoint_url)
        .force_path_style(force_path_style)
        .region(Region::new(region.unwrap_or(String::from("undefined"))))
        .build();
    Client::from_conf(config)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Cli::parse();

    let s3_client = setup_client(
        args.access_key_id,
        args.secret_access_key,
        args.endpoint_url.clone(),
        args.force_path_style,
        args.region,
    );

    let operation_id = Uuid::new_v4();
    println!(
        "Uploading objects to: {} in {} at {}",
        args.endpoint_url, args.bucket, operation_id
    );
    let response = match args.mode {
        // Send a standard request, customization only performed in order to print the request
        Mode::Standard => s3_client
            .put_object()
            .key(format!("{operation_id}/standard"))
            .bucket(args.bucket)
            .body(operation_id.to_string().into_bytes().into())
            .customize()
            .await
            .unwrap()
            .mutate_request(|request| println!("{request:?}")),
        // Send a modified request, removing the `x-id` query from the uri
        Mode::Modified => s3_client
            .put_object()
            .key(format!("{operation_id}/modified"))
            .bucket(args.bucket)
            .body(operation_id.to_string().into_bytes().into())
            .customize()
            .await
            .unwrap()
            .mutate_request(|request| {
                let uri = request.uri_mut();
                let query = uri
                    .query()
                    .unwrap()
                    .split('&')
                    .filter(|query_arg| !query_arg.starts_with("x-id"))
                    .collect::<Vec<_>>()
                    .concat();
                let path_and_query =
                    PathAndQuery::from_str(&format!("{}?{}", uri.path(), query)).unwrap();
                *uri = Uri::builder()
                    .authority(uri.authority().unwrap().to_owned())
                    .path_and_query(path_and_query)
                    .scheme(uri.scheme().unwrap().to_owned())
                    .build()
                    .unwrap();
                println!("{request:?}");
            }),
    }
    .send()
    .await;
    println!("Got: {response:?}");
    if response.is_err() {
        std::process::exit(-1)
    }
}
