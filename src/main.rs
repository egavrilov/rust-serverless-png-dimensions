use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  // dotenv::dotenv().ok();
  std::env::set_var(
    "RUST_LOG",
    "hyper_auth_server=debug,actix_web=info,actix_server=info",
  );
  std::env::set_var("RUST_BACKTRACE", "1");
  //  pretty_env_logger::init();

  let make_svc = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(lib::handle)) });

  let addr = ([127, 0, 0, 1], 7878).into();
  let server = Server::bind(&addr).serve(make_svc);

  println!("Listening on http://{}", addr);

  server.await?;

  Ok(())
}
