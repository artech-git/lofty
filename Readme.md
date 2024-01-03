
# Rust HTTP File Upload Handler

<!-- ![](./assest/image.png) -->
<p align="center">
<img src="./assest/image.png" width="150"/> 
&nbsp;
<img src="./assest/arrow.png" width="150"/> 
&nbsp;
<img src="./assest/data-server.png" width="150"/>
</p>

This project is designed to efficiently handle large HTTP file uploads, ranging from 6 to 14 gigabytes in size, while effectively scaling in high-traffic scenarios without overburdening file I/O operations. The system also ensures seamless reading of these uploaded files back through the HTTP endpoints. Built entirely in Rust programming language, utilizing the Tokio and Axum crates for high-performance asynchronous processing.

## Benefits

- **Scalability:** Capable of handling high volumes of traffic and concurrent file uploads without compromising performance.
- **Efficient Resource Utilization:** Optimized file I/O operations to prevent bottlenecks and efficiently manage system resources.
- **Asynchronous Processing:** Leverages Tokio's asynchronous runtime for efficient handling of large file uploads without blocking.
- **Safety and Reliability:** Rust's strong type system and memory safety features ensure robustness and reliability in operation.
- **HTTP Readback Capability:** Provides HTTP endpoints for effortless retrieval of uploaded files.

## Features

- **Large File Uploads:** Supports uploads ranging from 6 to 14 gigabytes in size.
- **Scalability:** Effectively scales to handle high-traffic scenarios without compromising performance.
- **Efficient I/O Handling:** Optimized file I/O operations to prevent bottlenecks.
- **HTTP Endpoint Readback:** Seamless retrieval of uploaded files via HTTP endpoints.

## Technologies Used

- **Rust:** The entire project is written in Rust, leveraging its performance and safety features.
- **Tokio:** Utilized for asynchronous I/O handling, aiding in efficient file uploads and downloads.
- **Axum:** Used for building high-performance web applications in Rust, facilitating HTTP handling.

## Installation

To use this project, ensure you have Rust installed. Clone the repository and use Cargo to build and run the application:

```bash
cargo build --release
cargo run --release
```
please update the settings.toml for fine tunning the project and adjusting the behaviour.

## Usage

1. Run the application using `cargo run --release`.
2. The server will start and listen for incoming HTTP file uploads on the designated endpoint(s).
3. Configure your client to make HTTP POST requests to upload files to the specified endpoint(s).

## Configuration

The application can be configured through environment variables or configuration files to set parameters such as:

- **Port:** Define the port the server listens on.
- **Maximum File Size:** Set the maximum allowed file size for uploads.
- **Concurrency Settings:** Configure the level of concurrent uploads or downloads the server can handle.

Refer to the `config.example.toml` file for an example configuration setup.

## Contributing

Contributions are welcome! If you'd like to contribute to this project.

## Acknowledgments

Special thanks to the Rust community, Tokio, Axum, and all contributors who help make this project possible.
