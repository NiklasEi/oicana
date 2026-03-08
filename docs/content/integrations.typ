The following libraries and packages integrate Oicana into different tech stacks. The usual functionality is
registration of a template and compilation to different output formats with given inputs.

For all integrations, you can find open source example applications on GitHub.

== Browser

https://www.npmjs.com/package/@oicana/browser

Oicana can run in browsers as WebAssembly. The `@oicana/browser` npm package contains a typed interface for interaction with the `.wasm` file.
To not block the UI, it's advisable to compile templates in a web worker.

An example application using Oicana in a React app #link("https://github.com/oicana/oicana-example-react/")[can be found on GitHub].

=== Initializing the WASM file

Oicana's WebAssembly file has to be hosted as part of your frontend application. The initialization method expects the path to the hosted file.
If your bundler supports it, the easiest way to get that URL is via `import wasmUrl from '@oicana/browser-wasm/oicana_browser_wasm_bg.wasm?url'`.

== C#sym.hash

https://www.nuget.org/packages/Oicana

The nuget package `Oicana` has a native interface to work with Oicana templates from C#sym.hash.

An example ASP.NET project using the package #link("https://github.com/oicana/oicana-example-asp-net/")[can be found on GitHub].

== Java

The `com.oicana:oicana` Maven package provides a native JNI interface to work with Oicana templates from Java. It uses native bindings for optimal performance on the server.

In addition to the main package, you need to add the native dependency for your target platform(s):

#table(
  columns: (auto, auto),
  [*Platform*], [*Artifact*],
  [Linux x86_64], [`com.oicana:oicana-linux-x86_64`],
  [Linux aarch64], [`com.oicana:oicana-linux-aarch64`],
  [macOS x86_64], [`com.oicana:oicana-macos-x86_64`],
  [macOS aarch64], [`com.oicana:oicana-macos-aarch64`],
  [Windows x86_64], [`com.oicana:oicana-windows-x86_64`],
)

For example, in Gradle:

```kotlin
dependencies {
    implementation("com.oicana:oicana:0.1.0-alpha.1")
    runtimeOnly("com.oicana:oicana-linux-x86_64:0.1.0-alpha.1")
}
```

You can add multiple native dependencies if your team uses different platforms. Only the matching native library will be loaded at runtime.

An example Spring Boot application using the package #link("https://github.com/oicana/oicana-example-java-spring-boot/")[can be found on GitHub].

== Node.js

https://www.npmjs.com/package/@oicana/node

The npm package `@oicana/node` provides a native Node.js interface to work with Oicana templates. It uses native bindings for optimal performance on the server.

An example NestJS application using the package #link("https://github.com/oicana/oicana-example-nestjs/")[can be found on GitHub].

== Rust

https://crates.io/crates/oicana

The `oicana` crate allows you to compile Oicana templates directly in Rust projects. This integration provides the most direct access to Oicana's core functionality.

An example Axum application using this crate #link("https://github.com/oicana/oicana-example-axum/")[can be found on GitHub].

== Python

https://pypi.org/project/oicana/

The `oicana` Python package provides native bindings to work with Oicana templates from Python. It uses native extensions for optimal performance.

An example FastAPI application using the package #link("https://github.com/oicana/oicana-example-fastapi/")[can be found on GitHub].

== PHP

The `oicana/oicana` Composer package provides a native PHP extension to work with Oicana templates. It uses native bindings for optimal performance.

An example PHP Slim application using the package #link("https://github.com/oicana/oicana-example-php-slim/")[can be found on GitHub].
