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
  use axum::{
      Router,
      body::Body,
      http::{StatusCode, header},
      response::{IntoResponse, Response},
      routing::post,
  };
  use oicana::Template;
  use oicana_export::pdf::export_merged_pdf;
  use oicana_input::{CompilationConfig, TemplateInputs};

  #[tokio::main]
  async fn main() {
      let app = Router::new()
          .route("/compile", post(compile));

      let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
          .await
          .unwrap();

      println!("Server running at http://127.0.0.1:3000");
      axum::serve(listener, app).await.unwrap();
  }

  async fn compile() -> impl IntoResponse {
      // Load template
      let template_file = File::open("templates/example-0.1.0.zip")
          .expect("Failed to open template file");
      let mut template = Template::init(template_file)
          .expect("Failed to initialize template");

      // Compile with development mode (uses development fallback values for inputs)
      let mut inputs = TemplateInputs::new();
      inputs.with_config(CompilationConfig::development());

      let result = template.compile(inputs)
          .expect("Failed to compile template");

      // Export to PDF
      let pdf = export_merged_pdf(&result.document, &template)
          .expect("Failed to export PDF");

      // Return PDF response
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

  This code defines a new POST endpoint at `/compile`. For every request, it loads the template, compiles it with an empty input list, and returns the PDF file. We use `CompilationConfig::development()` so the template uses the development value you defined for the `info` input ("Chuck Norris").

\
Start the service with `cargo run` and test the endpoint. You can use curl to download the PDF:

```bash
curl -X POST http://127.0.0.1:3000/compile --output example.pdf
```

The generated `example.pdf` file should contain your template with the development default value.

== About performance

The first compilation might take slightly longer than subsequent ones due to initialization overhead. However, PDF generation should typically take only a few milliseconds per request.

\
For production use, consider loading and caching the template once at startup rather than reading it from disk on every request. The #link("https://github.com/oicana/oicana-example-axum/")[open source Axum example project on GitHub] demonstrates this approach using a `DashMap` for thread-safe template caching.

== Passing inputs from Rust

Now let's use the template with the inputs you defined in the previous chapter. First, make sure to update the packed template in your Rust project. Run `oicana pack` in the template directory and replace `example-0.1.0.zip` in the Rust project with the new file.

\
Our `compile` function currently calls `template.compile(inputs)` with only a compilation config. This compiles the template without any explicit inputs. Let's add the name input you defined earlier.

\
Change the endpoint to set the input value, which allows us to compile in production mode:

#code(
  "Part of src/main.rs",
  ```rust
  async fn compile() -> impl IntoResponse {
      // ... template loading code from before

      // Prepare inputs
      let mut inputs = TemplateInputs::new();
      inputs.with_config(CompilationConfig::production());

      // Add JSON input
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
Notice that we switched to `CompilationConfig::production()` now that we're providing explicit input values. In production mode, the template will never fall back to development defaults. If no input value is provided, your Typst code will have to handle `none` values or the compilation will fail.

\
Calling the endpoint now will result in a PDF with "Baby Yoda" instead of "Chuck Norris". Building on this minimal service, you could set input values based on database entries or the request payload. Take a look at the #link("https://github.com/oicana/oicana-example-axum/")[open source Axum example project on GitHub] for a more complete showcase of the Oicana Rust integration, including blob inputs, error handling, and OpenAPI documentation.
