use warp::Filter;

pub async fn server(port: u16) {
    let wiki = warp::fs::dir("public");
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let routes = wiki.or(static_files);
    println!("Starting Server at: http://0.0.0.0:{}", port);
    warp::serve(routes).run(([0,0,0,0], port)).await;
}
