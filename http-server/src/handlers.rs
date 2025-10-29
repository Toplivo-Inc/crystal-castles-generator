use core::{config, processor};

use anyhow::Result;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response, StatusCode};

pub async fn generate(req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
    let body = req.collect().await?.to_bytes();
    let body_string = String::from_utf8(body.to_vec())?;

    let config: config::Config = serde_json::from_str(&body_string).unwrap();
    let processed_image = processor::ImageProcessor::process(&config).unwrap();
    processor::ImageProcessor::save_image(&processed_image, &config.output).unwrap();

    let mut res = Response::new(Full::new(Bytes::new()));
    *res.status_mut() = StatusCode::OK;
    Ok(res)
}
