The following libraries and packages integrate Oicana into different tech stacks. The usual functionality is
registration of a template and compilation to different output formats with given inputs.

For all integrations, you can find open source example applications on GitHub.

== Browser

https://www.npmjs.com/package/@oicana/browser

Oicana can run in browsers as WebAssembly. The `@oicana/browser` npm package contains a typed interface for interaction with the `.wasm` file.
To not blobk the UI, it's advisable to compile templates in a web worker.

An example application using Oicana in a React app #link("https://github.com/oicana/oicana-example-react/")[can be found on GitHub].

=== Initializing the WASM file

Oicana's WebAssembly file has to be hosted as part of your frontend application. The initialization method expects the path to the hosted file.
If your bundler supports it, the easiest way to get that URL is via `import wasmUrl from '@oicana/browser-wasm/oicana_browser_wasm_bg.wasm?url'`.

== C#sym.hash

https://www.nuget.org/packages/Oicana

The nuget package `Oicana` has a native interface to work with Oicana templates from C#sym.hash.

An example ASP.NET project using the package #link("https://github.com/oicana/oicana-example-asp-net/")[can be found on GitHub].

== Work in progress integrations

Integrations for Node and Rust are currently in development. If you are interested in using Oicana from other programming languages or environments, please let us know.
