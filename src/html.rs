//! 最小 HTML 解析引擎 —— 标签查找 + 实体解码 + 文本提取

/// 检查标签的 class 属性中是否包含指定值
fn tag_has_class(open_tag: &str, class: &str) -> bool {
    let mut pos = 0;
    while let Some(start) = open_tag[pos..].find("class=\"") {
        let val_start = pos + start + 7;
        if let Some(val_end) = open_tag[val_start..].find('"') {
            let values = &open_tag[val_start..val_start + val_end];
            if values.split_whitespace().any(|c| c == class) {
                return true;
            }
            pos = val_start + val_end + 1;
        } else {
            break;
        }
    }
    false
}

/// 检查标签的 id 属性是否匹配
fn tag_has_id(open_tag: &str, id: &str) -> bool {
    open_tag.contains(&format!("id=\"{id}\"")) || open_tag.contains(&format!("id='{id}'"))
}

/// 检查开标签是否匹配属性提示：
/// - `.classname` → class 属性
/// - `#id` → id 属性
/// - 裸字符串 → 原始子串匹配
pub fn attr_matches(open_tag: &str, hint: &str) -> bool {
    if let Some(class) = hint.strip_prefix('.') {
        tag_has_class(open_tag, class)
    } else if let Some(id) = hint.strip_prefix('#') {
        tag_has_id(open_tag, id)
    } else {
        open_tag.contains(hint)
    }
}

/// 在 HTML 中查找匹配标签的内容。能正确处理同标签嵌套。
/// `attr_hint`: `.class` / `#id` / 裸字符串
pub fn find_tags<'a>(html: &'a str, tag: &str, attr_hint: &str) -> Vec<&'a str> {
    let mut results = Vec::new();
    let open_marker = format!("<{tag}");
    let close_marker = format!("</{tag}>");
    let bytes = html.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        let rest = &bytes[pos..];
        let tag_start = match rest
            .windows(open_marker.len())
            .position(|w| w == open_marker.as_bytes())
        {
            Some(i) => pos + i,
            None => break,
        };

        let tag_end = match bytes[tag_start..].iter().position(|&b| b == b'>') {
            Some(i) => tag_start + i,
            None => break,
        };

        let open_tag = &html[tag_start..=tag_end];

        if !attr_hint.is_empty() && !attr_matches(open_tag, attr_hint) {
            pos = tag_end + 1;
            continue;
        }

        if bytes[tag_end - 1] == b'/' {
            results.push("");
            pos = tag_end + 1;
            continue;
        }

        let mut depth = 1u32;
        let mut search = tag_end + 1;

        while depth > 0 && search < bytes.len() {
            let tail = &bytes[search..];
            let next_open = tail
                .windows(open_marker.len())
                .position(|w| w == open_marker.as_bytes());
            let next_close = tail
                .windows(close_marker.len())
                .position(|w| w == close_marker.as_bytes());

            let close_first = match (next_open, next_close) {
                (None, Some(_)) => true,
                (Some(o), Some(c)) => c <= o,
                _ => false,
            };

            if close_first {
                if let Some(ci) = next_close {
                    depth -= 1;
                    if depth == 0 {
                        results.push(&html[tag_end + 1..search + ci]);
                        pos = search + ci + close_marker.len();
                        break;
                    }
                    search += ci + close_marker.len();
                }
            } else if let Some(oi) = next_open {
                depth += 1;
                search += oi + open_marker.len();
            } else {
                break;
            }
        }
        if depth == 0 {
            continue;
        }
        pos = tag_end + 1;
    }

    results
}

/// 去除 HTML 标签，解码常见实体，合并空白。
pub fn strip_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        if ch == '<' {
            in_tag = true;
            out.push(' ');
            continue;
        }
        if ch == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            out.push(ch);
        }
    }
    let out = out
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_removes_tags_and_decodes() {
        assert_eq!(strip_html("<p>hello &amp; world</p>"), "hello & world");
        assert_eq!(strip_html("  <b>bold</b>  "), "bold");
        assert_eq!(strip_html("a&#39;b"), "a'b");
    }

    #[test]
    fn find_tags_nested_divs() {
        let html = "<div class=\"a\"><div class=\"b\">inner</div></div>";
        let outer = find_tags(html, "div", ".a");
        assert_eq!(outer.len(), 1);
        assert!(outer[0].contains("class=\"b\""));
    }
}
