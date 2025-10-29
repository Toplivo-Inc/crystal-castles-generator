use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use anyhow::Result;
use async_native_tls::TlsAcceptor;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use macro_rules_attribute::apply;
use smol::{Async, Executor};
use smol_hyper::rt::{FuturesIo, SmolTimer};
use smol_macros::main;

mod stream;
use stream::SmolStream;

mod handlers;

/// Serves a request and returns a response.
async fn serve(req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
    println!("Serving {}", req.uri());
    match (req.method(), req.uri().to_string().as_str()) {
        (&Method::POST, "/api/v1/generate") => return handlers::generate(req).await,
        _ => {
            let mut res = Response::new(Full::new(Bytes::new()));
            *res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        },
    }
}

/// Handle a new client.
async fn handle_client(client: Async<TcpStream>, tls: Option<TlsAcceptor>) -> Result<()> {
    // Wrap it in TLS if necessary.
    let client = match &tls {
        None => SmolStream::Plain(client),
        Some(tls) => {
            // In case of HTTPS, establish a secure TLS connection.
            SmolStream::Tls(tls.accept(client).await?)
        }
    };

    // Build the server.
    hyper::server::conn::http1::Builder::new()
        .timer(SmolTimer::new())
        .serve_connection(FuturesIo::new(client), service_fn(serve))
        .await?;

    Ok(())
}

/// Listens for incoming connections and serves them.
async fn listen(
    ex: &Arc<Executor<'static>>,
    listener: Async<TcpListener>,
    tls: Option<TlsAcceptor>,
) -> Result<()> {
    // Format the full host address.
    let host = &match tls {
        None => format!("http://{}", listener.get_ref().local_addr()?),
        Some(_) => format!("https://{}", listener.get_ref().local_addr()?),
    };
    println!("Listening on {}", host);

    loop {
        // Wait for a new client.
        let (client, _) = listener.accept().await?;

        // Spawn a task to handle this connection.
        ex.spawn({
            let tls = tls.clone();
            async move {
                if let Err(e) = handle_client(client, tls).await {
                    println!("Error while handling client: {}", e);
                }
            }
        })
        .detach();
    }
}

#[apply(main!)]
async fn main(ex: &Arc<Executor<'static>>) -> Result<()> {
    // Initialize TLS with the local certificate, private key, and password.
    // let identity = Identity::from_pkcs12(include_bytes!("identity.pfx"), "password")?;
    // let tls = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity)?);

    // Start HTTP and HTTPS servers.
    let http = listen(
        ex,
        Async::<TcpListener>::bind(([127, 0, 0, 1], 8000))?,
        None,
    );
    // let https = listen(
    //     ex,
    //     Async::<TcpListener>::bind(([127, 0, 0, 1], 8001))?,
    //     Some(tls),
    // );
    // future::try_zip(http, https).await?;
    http.await?;
    Ok(())
}
