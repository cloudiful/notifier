use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use cloudiful_notifier_core::{DeliveryChannel, MessageEnvelope, NotifierError};
use serde_json::json;

use crate::WebhookChannel;

fn spawn_server(response: &'static str) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        let mut buffer = Vec::with_capacity(8192);
        let mut chunk = [0; 1024];

        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(size) => buffer.extend_from_slice(&chunk[..size]),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("failed to read request: {error}"),
            }
        }

        sender
            .send(String::from_utf8_lossy(&buffer).to_string())
            .unwrap();
        stream.write_all(response.as_bytes()).unwrap();
    });

    (address, receiver)
}

#[tokio::test]
async fn webhook_sends_generic_json_payload() {
    let (url, receiver) =
        spawn_server("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    let mut extra_headers = BTreeMap::new();
    extra_headers.insert("x-trace-id".to_string(), "abc".to_string());
    let channel = WebhookChannel {
        url,
        bearer_token: Some("token-123".to_string()),
        extra_headers,
    };
    let mut message = MessageEnvelope::new("plain body").with_title("Important");
    message
        .metadata
        .insert("severity".to_string(), json!("critical"));

    let result = channel
        .deliver(&reqwest::Client::new(), &message)
        .await
        .unwrap();
    let request = receiver.recv().unwrap();
    let request_lower = request.to_ascii_lowercase();

    assert_eq!(result.http_status, Some(200));
    assert!(request.starts_with("POST / HTTP/1.1"));
    assert!(request_lower.contains("authorization: bearer token-123"));
    assert!(request_lower.contains("x-trace-id: abc"));
    assert!(request.contains("\"title\":\"Important\""));
    assert!(request.contains("\"body\":\"plain body\""));
    assert!(request.contains("\"metadata\":{\"severity\":\"critical\"}"));
}

#[tokio::test]
async fn webhook_rejects_reserved_headers() {
    let mut extra_headers = BTreeMap::new();
    extra_headers.insert("Authorization".to_string(), "override".to_string());
    let channel = WebhookChannel {
        url: "https://example.com".to_string(),
        bearer_token: None,
        extra_headers,
    };
    let message = MessageEnvelope::new("body");

    let error = channel
        .deliver(&reqwest::Client::new(), &message)
        .await
        .unwrap_err();

    assert_eq!(
        error,
        NotifierError::ReservedHeader {
            header: "Authorization".to_string(),
        }
    );
}
