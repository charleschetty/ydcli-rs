# ydcli-rs

有道词典命令行版 — 从 dict.youdao.com 查询单词释义、发音、词形变化和例句。

相比 [无道词典](https://github.com/ChestnutHeng/Wudao-dict)，本项目不使用 Server/Client 模式，内存占用更少，
输出更贴近无道词典的完整模式。

![](https://raw.githubusercontent.com/charleschetty/ydcli-rs/main/shots/1.png)

## 安装

```sh
cargo install --path .
```

## 用法

```sh
# 查询单词
ydcli <word>

# 查看帮助
ydcli --help

# 查看版本
ydcli --version
```

## 输出内容

- **释义** — 单词的中文释义
- **发音** — 英式/美式音标
- **词形变化** — 过去式、过去分词、复数等
- **例句** — 柯林斯词典例句及翻译

## Todo

- [ ] 简明模式
- [ ] 缓存
- [x] 支持中文查询
- [x] 零外部依赖

## 参考

[无道词典](https://github.com/ChestnutHeng/Wudao-dict) · [charcoal](https://github.com/LighghtEeloo/charcoal)

---

> **AI 声明**：`043c228`（fix head issue）之后的所有 commit 均由 Claude (Anthropic) 生成。
