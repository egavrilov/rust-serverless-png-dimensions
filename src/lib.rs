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
    (&Method::GET, "/") => Ok(handle_service(req).await.unwrap()),
    (&Method::GET, "/favicon.ico") | _ => {
      let mut not_found = Response::default();
      *not_found.status_mut() = StatusCode::NOT_FOUND;
      Ok(not_found)
    }
  }
}

async fn handle_service(_req: Request<Body>) -> Result<Response<Body>, reqwest::Error> {
  let image_url = match get_image_url(_req).await? {
    Some(url) => url,
    None => return Ok(error_response(
        "No URL parameter set, please use function_path/?url=http://domain/path/to/image.png".to_string(),
        StatusCode::BAD_REQUEST
      )
    )
  };

  match get_png_size(image_url).await? {
    Some(dimensions) => {
      let json = serde_json::to_string(&dimensions).unwrap();
      return Ok(success_response_json(json))
    },
    _ => {
      let response = error_response(
        "Currently only PNG format is supported".to_string(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
      );
      return Ok(response)
    }
  };
}

fn success_response_json(json: String) -> Response<Body> {
  Response::builder()
    .header("Content-Type", "application/json")
    .body(Body::from(json))
    .unwrap()
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

async fn get_image_url(_req: Request<Body>) -> Result<Option<String>, reqwest::Error> {
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
    Some(url) if url.is_empty() => None,
    Some(url) => Some(url.to_string()),
    _ => None
  })
}

async fn get_png_size(url: String) -> Result<Option<Dimensions>, reqwest::Error> {
  let mut response = reqwest::get(&url).await?;
  let bytes = match response.chunk().await? {
    Some(bytes) => bytes,
    None => return Ok(None)
  };
  let is_png = bytes.starts_with(b"\x89PNG\r\n\x1a\n");

  if !is_png {
    return Ok(None)
  }

  let width = bytes.slice(16..20);
  let height = bytes.slice(20..24);
  let dimensions = Dimensions { width: to_big_int(width), height: to_big_int(height) };
  Ok(Some(dimensions))
}

fn to_big_int(bytes: bytes::Bytes) -> u32 {
  return BigEndian::read_u32(&*bytes);
}
