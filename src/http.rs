//! 最小 HTTP 客户端 —— 纯 TCP 实现

use crate::Result;
use std::io::{Read, Write};
use std::net::TcpStream;

/// URL 编码：保留 `A-Z a-z 0-9 - _ . ~`，其余按 UTF-8 字节做 `%XX` 编码
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

/// 用原生 TCP 发送 HTTP GET 请求，返回响应体
pub fn http_get(host: &str, path: &str) -> Result<String> {
    let mut stream = TcpStream::connect((host, 80))
        .map_err(|e| format!("无法连接到 {host}：{e}"))?;

    let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("发送请求失败：{e}"))?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取响应失败：{e}"))?;

    let response = String::from_utf8_lossy(&buf);
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    Ok(body.to_owned())
}
