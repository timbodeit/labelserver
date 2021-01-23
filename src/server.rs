use hyper::body::Buf;
use hyper::{Body, Request, Response, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use std::convert::Infallible;

/// A handler for "/" page.
async fn home_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Try POSTing data to /print/<labelname> such as: `curl localhost:3000/print/simple -XPOST -d '{\"content\": \"Hello World\"}'`")))
}

/// A handler for "/print/:template_name" page.
async fn print_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let now = std::time::Instant::now();
    let template_name = req.param("template_name").unwrap().to_owned();

    let whole_body = hyper::body::aggregate(req).await.unwrap();
    let data: serde_json::Value = serde_json::from_reader(whole_body.reader()).unwrap();

    let pdf = crate::generator::make_label(&template_name, &data).unwrap();
    crate::print::print_pdf(pdf).unwrap();

    let message = format!(
        "Made label {} in {:.2}s",
        template_name,
        now.elapsed().as_secs_f32()
    );
    log::info!("{}", &message);

    Ok(Response::new(Body::from(message)))
}

async fn error_handler(err: routerify::Error, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

// A middleware which logs an http request.
async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}

pub fn router() -> RouterService<Body, Infallible> {
    RouterService::new(
        Router::builder()
            .middleware(Middleware::pre(logger))
            .get("/", home_handler)
            .post("/print/:template_name", print_handler)
            .err_handler_with_info(error_handler)
            .build()
            .unwrap(),
    )
    .unwrap()
}
