# Simple Web Server in Rust

This program is a web server that runs on a given IP address/port, and responds to with an HTML page to render in a browser, but could theoretically be used to respond in a variety of ways, not just HTML pages.

The web server uses a ThreadPool implementation to gain concurrent requests, improving performance espeically when there are long running requests, thus not holding up subsequent request that might not take as long.

This program was built by following the tutorial in the "[Rust Programming Language Book (2021 Edition)](https://nostarch.com/rust-programming-language-2nd-edition)" book, from Chapter 20: Building a Multithreaded Web Server

- [Online Tutorial](https://doc.rust-lang.org/stable/book/ch20-00-final-project-a-web-server.html)

## How to Use

#### 1. Build & Run

To gain access to the program from the source code, you can run the following command to build the Rust source code into a binary/executable that can be run from the command line. Then, since there are no command line arguments for this program as of now, we can simply call cargo run after to start the server on locahost:

```
cargo build
cargo run
```
