use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use web_server_rust::ThreadPool;

fn main() {
    // Listen for any TCP connections coming into our program by using the TcpListener
    // and "binding" to a particular IP address/port
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Create a ThreadPool with a set number of threads so we can handle requests
    // coming into our server in a multi-threaded/concurrent way
    let pool = ThreadPool::new(4);

    // Loop over the "incoming" stream data from the listener above
    // Each item in the iterator is a "possible" connection, so we have to keep looping
    // until we successfully receive the connection
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // At this point, the connection has been established, so we'll take the stream
        // and respond back appropriately to the incoming request with a valid HTTP/TCP response
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    // Now, WE have to assemble the request and return it back to the user/requester
    // in the form of a valid HTTP response, using the stream.write_all function

    // To do so, we'll render a simple HTML page by reading in the contents of an HTML
    // page and passing the contents of that HTML as the body of the response returned
    // to the user/stream

    // First, created a BufReader, so we can get a way to receive the data from the stream
    let reader = BufReader::new(&mut stream);

    // Then, read off the first line from the request, which will be in the form:
    //    "Method Uri HttpVersion" -> i.e.: "GET / HTTP/1.1"
    let request_line = reader.lines().next().unwrap().unwrap();

    // We can validate that this is a valid URI/request, or at least one we're able to handle,
    // and then assemble an HTTP response to write to the stream object
    //   Response: "HttpVersion StatusCode Reason-Phrase\n headers\n response-body"
    //   Example:  "HTTP/1.1 200 OK\n\n" = Ok Response with no response body (nothing returned to user)
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "pages/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "pages/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "pages/404.html"),
    };

    // Lastly, we'll take that assembled HTTP response...
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!(
        "{status_line}\r\n\
        Content-Length: {length}\r\n\r\n\
        {contents}"
    );

    // ...and send it back to the user/requester using the stream.write_all function
    stream.write_all(response.as_bytes()).unwrap();
}
