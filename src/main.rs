use std::convert::Infallible;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method};
use image::{GenericImageView, ColorType, imageops::FilterType};
use std::io::BufWriter;
use std::borrow::Cow;
use std::time::Instant;

#[derive(Debug)]
struct ThumbOptions {
    url: String,
    width: u32
}

impl ThumbOptions {
    fn new(opts: HashMap<String, String>) -> ThumbOptions {
        let url: String = match opts.get("url") {
            Some(val) => String::from(val),
            None => String::from("")
        };

        let width: u32 = match opts.get("width") {
            Some(val) => val.parse::<u32>().unwrap(),
            None => 180
        };

        ThumbOptions {
            url: url,
            width: width
        }
    }
}

fn querify(string: &str) -> HashMap<String, String> {
    let mut acc: HashMap<String, String> = HashMap::new();
    let pairs: Vec<&str> = string.split('&').collect();
    for kv in pairs {
        let mut it = kv.splitn(2, '=').take(2);
        match (it.next(), it.next()) {
            (Some("url"), Some(v)) => acc.insert(String::from("url"), v.to_string()),
            (Some("width"), Some(v)) => acc.insert(String::from("width"), v.to_string()),
            _ => continue,
        };
    }
    acc
}

impl From<&str> for ThumbOptions {
    fn from(query_params: &str) -> Self {
        let qs = querify(query_params);
        ThumbOptions::new(qs)
    }
}

async fn handle_thumbnail(opts: ThumbOptions, client: reqwest::Client) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    let download_start = Instant::now();

    let file = client.get(&opts.url)
        .send()
        .await
        .expect("Failed sending request")
        .bytes()
        .await
        .expect("Bytes unwrap err");
    
    #[cfg(debug_assertions)]
    let download_duration = download_start.elapsed();
    #[cfg(debug_assertions)]
    println!("Download duration is {:?}", download_duration);

    #[cfg(debug_assertions)]
    let render_start = Instant::now();

    let width = opts.width;
    let image = image::load_from_memory_with_format(&file, image::ImageFormat::Png).unwrap();
    let original_width = image.width();
    let ratio = original_width / width;
    let original_height = image.height();
    let height = original_height / ratio;

    let resized = image::imageops::resize(&image, width, height, FilterType::Nearest);
    let mut bytes: Vec<u8> = vec![];
    let fout = BufWriter::new(&mut bytes);
    image::png::PNGEncoder::new(fout).encode(&resized, width, height, ColorType::Rgba8).unwrap();

    #[cfg(debug_assertions)]
    let render_duration = render_start.elapsed();
    #[cfg(debug_assertions)]
    println!("Render duration {:?}", render_duration);

    Ok(bytes)
}

async fn router(req: Request<Body>, client: reqwest::Client) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();

    match (req.method(), uri.path()) {
        (&Method::GET, "/thumbnail") => {
            let q = uri.query().unwrap();

            let thumb = handle_thumbnail(ThumbOptions::from(q), client)
                .await
                .expect("Handling did not go that well");

            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(thumb))
                .unwrap();

            Ok(response)
        },
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client: reqwest::Client = reqwest::Client::new();
    let cow_client: Cow<reqwest::Client> = Cow::Owned(client);

    let make_svc = make_service_fn(|_conn| {
        let cow_client = cow_client.clone();
        async { 
            let clone = cow_client.into_owned();

            Ok::<_, Infallible>(service_fn(move |req| {
                router(req, clone.to_owned())
            })) 
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}