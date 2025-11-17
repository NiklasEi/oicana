#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *

In this chapter, you'll integrate Oicana into a Rust web service using #link("https://github.com/tokio-rs/axum")[axum]. axum is a web application framework built on top of #link("https://tokio.rs/")[Tokio] and #link("https://github.com/tower-rs/tower")[Tower], designed for building fast, reliable HTTP services. We'll create a simple async web service that compiles your Oicana template to PDF and serves it via an HTTP endpoint.

\
#note[This section assumes that you have a working Rust setup with cargo. If that is not the case, please follow #link("https://www.rust-lang.org/tools/install")[the official Rust installation guide] to install Rust on your machine.]

\
Let's start with a fresh Axum project. First, create a new binary project with `cargo init --bin` in a new directory. Then add the necessary dependencies to your `Cargo.toml`:

#code("Part of Cargo.toml", ```toml
[dependencies]
oicana = "0.1.0-alpha.5"
oicana_files = "0.1.0-alpha.5"
oicana_input = "0.1.0-alpha.5"
oicana_export = "0.1.0-alpha.5"

axum = { version = "0.8", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
serde_json = "1.0"
```)

\
Run `cargo build` to download and compile the dependencies. This might take a few minutes on first run.

== New service endpoint

We will define a new endpoint to compile our Oicana template to a PDF and return the PDF file to the user.

\
1. Create a new directory in the Rust project called `templates` and copy `example-0.1.0.zip` into that directory.
2. Replace the contents of `src/main.rs` with a basic Axum server that loads and compiles the template:

  \
  #code("src/main.rs", ```rust
  use std::fs::File;
  use std::sync::{Arc, Mutex};
  use axum::{
      Router,
      body::Body,
      extract::State,
      http::{StatusCode, header},
      response::{IntoResponse, Response},
      routing::post,
  };
  use oicana::Template;
  use oicana_export::pdf::export_merged_pdf;
  use oicana_input::{CompilationConfig, TemplateInputs};

  #[tokio::main]
  async fn main() {
      let template_file = File::open("templates/example-0.1.0.zip")
          .expect("Failed to open template file");
      let template = Template::init(template_file)
          .expect("Failed to initialize template");

      let template = Arc::new(Mutex::new(template));

      let app = Router::new()
          .route("/compile", post(compile))
          .with_state(template);

      let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
          .await
          .unwrap();

      println!("Server running at http://127.0.0.1:3000");
      axum::serve(listener, app).await.unwrap();
  }

  async fn compile(State(template): State<Arc<Mutex<Template<PackedTemplate>>>>) -> impl IntoResponse {
      let mut template = template.lock().unwrap();

      // Compile with development mode for demonstration
      // (uses development fallback values for inputs)
      let mut inputs = TemplateInputs::new();
      inputs.with_config(CompilationConfig::development());

      let result = template.compile(inputs)
          .expect("Failed to compile template");

      let pdf = export_merged_pdf(&result.document, &*template)
          .expect("Failed to export PDF");

      Response::builder()
          .status(StatusCode::OK)
          .header(header::CONTENT_TYPE, "application/pdf")
          .header(
              header::CONTENT_DISPOSITION,
              "attachment; filename=\"example.pdf\"",
          )
          .body(Body::from(pdf))
          .unwrap()
  }
  ```)

  This code loads the template once at startup and wraps it in `Arc<Mutex<Template<PackedTemplate>>>`. The `Arc` (Atomic Reference Counted pointer) allows sharing across threads, while `Mutex` provides the mutable access needed by `compile()`. When parallel requests come in, they share the same template - each request locks the mutex (one at a time), compiles, then releases the lock.

  \
  The `/compile` endpoint compiles the template and returns a PDF. We explicitly use `CompilationConfig::development()` here to demonstrate how the template uses the development value you defined for the `info` input ("Chuck Norris"). We will set an input value in a later step.

\
Start the service with `cargo run` and test the endpoint. You can use curl to download the PDF:

```bash
curl -X POST http://127.0.0.1:3000/compile --output example.pdf
```

The generated `example.pdf` file should contain your template with the development default value.

== About performance

PDF generation should typically take only a few milliseconds per request. Since we're loading the template once at startup and sharing it via `Arc`, there's no file I/O overhead on subsequent requests.

\
For managing multiple templates, the #link("https://github.com/oicana/oicana-example-axum/")[open source Axum example project on GitHub] demonstrates using a `DashMap` for thread-safe template caching.

== Passing inputs from Rust

Now let's use the template with the inputs you defined in the previous chapter. First, make sure to update the packed template in your Rust project. Run `oicana pack` in the template directory and replace `example-0.1.0.zip` in the Rust project with the new file.

\
Our `compile` function currently does not set a value for the template input. Since we use `CompilationConfig::development()`, the development value of `{ "name": "Chuck Norris" }` is used. Now we'll provide an explicit input value and switch to production mode:

#code(
  "Part of src/main.rs",
  ```rust
  async fn compile(State(template): State<Arc<Mutex<Template<PackedTemplate>>>>) -> impl IntoResponse {
      let mut template = template.lock().unwrap();

      let mut inputs = TemplateInputs::new();
      inputs.with_config(CompilationConfig::production());

      let json_value = serde_json::json!({ "name": "Baby Yoda" });
      inputs.with_input(
          oicana_input::input::json::JsonInput::new(
              "info".to_string(),
              json_value.to_string(),
          )
      );

      let result = template.compile(inputs)
          .expect("Failed to compile template");

      // ... PDF export and response code from before
  }
  ```,
)

\
Notice that we switched to `CompilationConfig::production()` now that we're providing explicit input values. Production mode is the recommended default for all document compilation in your application - it ensures you never accidentally generate a document with test data. In production mode, the template will never fall back to development values for inputs. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles `none` values for that input.

\
Calling the endpoint now will result in a PDF with "Baby Yoda" instead of "Chuck Norris". Building on this minimal service, you could set input values based on database entries or the request payload. Take a look at the #link("https://github.com/oicana/oicana-example-axum/")[open source Axum example project on GitHub] for a more complete showcase of the Oicana Rust integration, including blob inputs, error handling, and OpenAPI documentation.
