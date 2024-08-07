use axum::extract::Request;
use axum::http::Uri;

pub fn normalize_path<B>(mut request: Request<B>) -> Request<B> {
  let uri = request.uri_mut();

  let mut parts = uri.clone().into_parts();
  if let Some(path_and_query) = parts.path_and_query {
    let normalized_path = do_normalize_path(path_and_query.as_str());
    parts.path_and_query = Some(
      normalized_path
        .parse()
        .expect(&format!("failed to parse path: {:?}", normalized_path)),
    );
    *uri = Uri::from_parts(parts).expect("failed to reconstruct uri");
  }
  request
}

fn do_normalize_path(path: &str) -> String {
  let mut normalized_path = String::new();
  let mut prev_was_slash = false;
  let mut query_part = false;

  for ch in path.chars() {
    if ch == '?' {
      query_part = true;
    }

    if query_part {
      normalized_path.push(ch);
      continue;
    }

    if ch == '/' {
      if !prev_was_slash {
        normalized_path.push(ch);
      }
      prev_was_slash = true;
    } else {
      normalized_path.push(ch);
      prev_was_slash = false;
    }
  }

  normalized_path
}
