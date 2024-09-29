use axum::{
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::{self, Stream};
use rand::Rng;
use std::time::Duration;
use tokio::{net::TcpListener, time::sleep};

#[tokio::main]
async fn main() {
    let router = Router::new().route("/stream", get(handle_stream));
    let addr = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    axum::serve(addr, router).await.unwrap();
}

/// This is the route handler that streams random words as Server-Sent Events (SSE)
async fn handle_stream() -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    Sse::new(generate_random_word_stream())
}

/// Function that generates a stream of random words with a delay
fn generate_random_word_stream() -> impl Stream<Item = Result<Event, axum::Error>> {
    let words = vec![
        "hello", "world", "axum", "rust", "async", "stream", "http", "server",
    ];

    // Create an asynchronous stream
    stream::unfold(0, move |count| {
        let words = words.clone();
        async move {
            if count < words.len() {
                // Simulate async delay of 1 second
                sleep(Duration::from_secs(1)).await;
                let random_word = words[rand::thread_rng().gen_range(0..words.len())].to_string();

                // Return the word as a server-sent event (SSE) event
                // Not sure yet if this is the best way to do it, but
                // for an minimal example this is probably fine.
                return Some((Ok(Event::default().data(random_word)), count + 1));
            }
            None // Ends the stream
        }
    })
}

#[tokio::test]
async fn test_stream_response() {
    let router = Router::new().route("/stream", get(handle_stream));

    let client = axum_test_helper::TestClient::new(router).await;

    // Send a GET request to the /stream endpoint.
    let mut response = client.get("/stream").send().await;

    assert_eq!(response.status(), 200);

    while let Some(chunk) = response.chunk().await {
        let text = String::from_utf8(chunk.to_vec()).unwrap();
        println!("Received stream chunk: {}", text);

        assert!(!text.is_empty());
    }
}
