use ::std::collections::HashMap;
use hyper::{Body, Request, Response, Method, StatusCode};
use serde::{Deserialize, Serialize};
use byteorder::{ByteOrder, BigEndian};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Dimensions {
  width: u32,
  height: u32
}

pub async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
  match (req.method(), req.uri().path()) {
    (&Method::GET, "/favicon.ico") => Ok(Response::new(Body::empty())),
    (&Method::GET, "/") => Ok(handle_service(req).await.unwrap()),
    _ => {
      let mut not_found = Response::default();
      *not_found.status_mut() = StatusCode::NOT_FOUND;
      Ok(not_found)
    }
  }
}

async fn handle_service(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
  let image_url = get_image_url(_req).await.unwrap();

  if image_url.is_empty() {
    let response = error_response(
      "No URL parameter set, please use function_path/?url=http://domain/path/to/image.png".to_string(),
      StatusCode::BAD_REQUEST
    );
    return Ok(response)
  }

  let dimensions = get_png_size(image_url).await.unwrap();

  if dimensions.width == 0 && dimensions.height == 0 {
    let response = error_response(
      "Currently only PNG format is supported".to_string(),
      StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    return Ok(response)
  }

  let json = serde_json::to_string(&dimensions);

  Ok(Response::builder()
    .header("Content-Type", "application/json")
    .body(Body::from(json.unwrap()))
    .unwrap())
}

fn error_response(message: String, status: StatusCode) -> Response<Body> {
  let json = json!({
    "error": { "message": &message }
  });

  Response::builder()
    .status(status)
    .header("Content-Type", "application/json") // ContentType::json() not there anymore
    .body(Body::from(json.to_string()))
    .unwrap()
}

async fn get_image_url(_req: Request<Body>) -> Result<String, hyper::Error> {
  let query_pairs = _req
    .uri()
    .query()
    .map(|v| {
        url::form_urlencoded::parse(v.as_bytes())
            .into_owned()
            .collect()
    })
    .unwrap_or_else(HashMap::new);

  Ok(match query_pairs.get("url") {
    Some(url) => url.to_string(),
    None => String::new()
  })
}

async fn get_png_size(url: String) -> Result<Dimensions, reqwest::Error> {
  let mut response = reqwest::get(&url).await?;
  let bytes = response.chunk().await?.unwrap();
  let is_png = bytes.starts_with(b"\x89PNG\r\n\x1a\n");

  if !is_png {
    return Ok(Dimensions { width: 0, height: 0 })
  }

  let width = bytes.slice(16..20);
  let height = bytes.slice(20..24);
  let dimensions = Dimensions { width: to_big_int(width), height: to_big_int(height) };
  Ok(dimensions)
}

fn to_big_int(bytes: bytes::Bytes) -> u32 {
  return BigEndian::read_u32(&*bytes);
}
