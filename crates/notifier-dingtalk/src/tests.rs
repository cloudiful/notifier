use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

use crate::core::{DeliveryChannel, MessageEnvelope, NotifierError};

use super::{DingtalkChannel, signing::sign};

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

#[test]
fn sign_produces_non_empty_output() {
    let value = sign("1715000000000", "SECabc").unwrap();

    assert!(!value.is_empty());
}

#[tokio::test]
async fn dingtalk_sends_title_and_body() {
    let (base_url, receiver) = spawn_server(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 27\r\nConnection: close\r\n\r\n{\"errcode\":0,\"errmsg\":\"ok\"}",
    );
    let channel = DingtalkChannel {
        webhook_url: format!("{base_url}/robot/send?access_token=abc"),
        secret: Some("SECabc".to_string()),
        keywords: vec!["ops".to_string()],
    };
    let message = MessageEnvelope::new("Body line").with_title("Critical");

    let result = channel
        .deliver(&reqwest::Client::new(), &message)
        .await
        .unwrap();
    let request = receiver.recv().unwrap();

    assert_eq!(result.http_status, Some(200));
    assert!(request.starts_with("POST /robot/send?access_token=abc&timestamp="));
    assert!(request.contains("\"content\":\"ops Critical\\nBody line\""));
}

#[tokio::test]
async fn dingtalk_rejects_too_many_keywords() {
    let channel = DingtalkChannel {
        webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=abc".to_string(),
        secret: None,
        keywords: (0..11).map(|index| format!("k{index}")).collect(),
    };
    let message = MessageEnvelope::new("Body line");

    let error = channel
        .deliver(&reqwest::Client::new(), &message)
        .await
        .unwrap_err();

    assert_eq!(
        error,
        NotifierError::InvalidMessage {
            provider: "dingtalk",
            message: "keywords cannot exceed 10".to_string(),
        }
    );
}
