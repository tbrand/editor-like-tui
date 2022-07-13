# editor-like-tui

This repository is totally for my practice. It doesn't help anything for most people. But if you try to build your editor by yourself in Rust, it could help a bit.

[![asciicast](https://asciinema.org/a/DEnjay9ThQQtm8iEQaz2feVnA.svg)](https://asciinema.org/a/DEnjay9ThQQtm8iEQaz2feVnA)

## How to use

Just execute as same as Rust project. After cloning this repository, type the command below.

```bash
cargo run
```

The tui has emasc-like key binds, but not compatible. Here is the list.
- **Ctrl-b** move left
- **Ctrl-f** move right
- **Ctrl-p** move up
- **Ctrl-n** move down
- **Ctrl-a** move front
- **Ctrl-e** move end
- **Ctrl-j** new line (enter)
- **Ctrl-h** backspace
- **Ctrl-d** delete
- **Ctrl-k** kill
- **Ctrl-y** paste
- **Ctrl-x** toggle "x mode" on

In "x mode", you can
- **[** move top
- **]** move bottom
- **2** split frame horizontally
- **3** split frame vertically
- **o** move cursor to the next frame
- **b** create new buffer
- **0** remove frame
- **Ctrl-c** close app

There is no function that save the text into files now.

## Development

An interesting part of this project is that this repository doesn't contains any unit tests at all for now. Instead, it has a monkey test that executes every commands randomly. You can try it by the command below.

```bash
cargo test monkey_test::run
```
