use std::sync::atomic::AtomicBool;

pub static mut APP_LIVE: AtomicBool = AtomicBool::new(true);

pub mod filters {
  use super::wrapped_handler;
  use crate::image::ThumbOptions;
  use warp::Filter;

  pub fn service(
    client: reqwest::Client,
  ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    thumbnail(client).or(info()).or(ready())
  }

  pub fn thumbnail(
    client: reqwest::Client,
  ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("thumbnail")
      .and(warp::get())
      .and(warp::query::<ThumbOptions>())
      .and(with_client(client))
      .and_then(wrapped_handler::wrapped_thumbnail)
  }

  fn with_client(
    client: reqwest::Client,
  ) -> impl Filter<Extract = (reqwest::Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
  }

  pub fn info() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("info")
      .and(warp::get())
      .and_then(wrapped_handler::info)
  }

  pub fn ready() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("ready")
      .and(warp::get())
      .and_then(wrapped_handler::ready)
  }
}

mod wrapped_handler {
  use crate::config::Config;
  use crate::image::{handle_thumbnail, ThumbOptions};
  use hyper::Body;
  use std::convert::Infallible;
  use std::sync::atomic::Ordering;
  use warp::http::Response;

  pub async fn wrapped_thumbnail(
    opts: ThumbOptions,
    client: reqwest::Client,
  ) -> Result<impl warp::Reply, Infallible> {
    let builder = Response::builder();
    let data = match handle_thumbnail(opts, client).await {
      Ok(data) => builder.status(http::StatusCode::OK).body(Body::from(data)),
      Err(err) => builder
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("{}", err))),
    };

    return Ok(data);
  }

  pub async fn info() -> Result<impl warp::Reply, Infallible> {
    let config = Config::get_current();
    let json_str: String =
      serde_json::to_string(&config.version).expect("Could not convert readiness struct to json");

    Ok(
      Response::builder()
        .header("Content-Type", "application/json")
        .header("content-length", json_str.len())
        .status(http::StatusCode::OK)
        .body(hyper::Body::from(json_str))
        .unwrap(),
    )
  }

  pub async fn ready() -> Result<impl warp::Reply, Infallible> {
    let app_live;
    unsafe {
      app_live = super::APP_LIVE.load(Ordering::Relaxed);
    }
    let status = match app_live {
      false => http::StatusCode::SERVICE_UNAVAILABLE,
      true => http::StatusCode::OK,
    };
    let response = Response::builder()
      .status(status)
      .body(Body::empty())
      .unwrap();
    Ok(response)
  }
}
