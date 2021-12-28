use anyhow::Context;

use std::path::Path;

pub fn launch_server(dir: &Path, open_browser: bool) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let address = [127, 0, 0, 1];
            let port = 8000;
            if open_browser {
                tokio::spawn(async move {
                    let url = format!(
                        "http://{}.{}.{}.{}:{}/preview",
                        address[0], address[1], address[2], address[3], port
                    );
                    if let Err(e) = open::that(url).context("failed to open browser") {
                        log::warn!("{}", e);
                    }
                });
            }

            let app = axum::Router::new().nest(
                "/",
                axum::routing::get_service(tower_http::services::ServeDir::new(&dir))
                    .handle_error(|e| {
                        async move {
                            (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Unhandled internal error: {}", e),
                            )
                        }
                    }),
            );
            axum::Server::bind(&std::net::SocketAddr::from((address, port)))
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
}
