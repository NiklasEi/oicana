#import "/src/boxes.typ": *
#import "/src/docs-link.typ": *


You've created a working Oicana template with a dynamic input! Now it's time to integrate it into an application.

== One Template, Multiple Platforms

A key strength of Oicana is that the exact same template works across all integrations. The `example-0.1.0.zip` file you created can be used in C#sym.hash, Node.js, Rust, Python, PHP, or browser environments.

\
Develop templates once and use them everywhere.

\
#alpha-note[More integrations are work in progress. If you are missing support for specific tech stacks, please reach out!]

== Available Integrations

The following chapters provide step-by-step guides for using your template with different programming languages and frameworks. You only need to follow one path to get started - pick the one that matches your tech stack.

\
If you're working on a multi-language project or want to compare approaches, feel free to explore multiple paths. Each guide is self-contained and uses the same template you created earlier.

=== C#sym.hash / ASP.NET

#link("4-1-csharp-integration.html")[Go to C#sym.hash Guide →]

\
#note[Prerequisites: .NET 8 or later installed on your machine]

=== Node.js / NestJS

#link("4-3-nodejs-integration.html")[Go to Node.js Guide →]

\
#note[Prerequisites: Node.js 18 or later installed on your machine]

=== Rust / Axum

#link("4-2-rust-integration.html")[Go to Rust Guide →]

\
#note[Prerequisites: Rust toolchain (cargo) installed on your machine]

=== Python / FastAPI

#link("4-4-python-integration.html")[Go to Python Guide →]

\
#note[Prerequisites: Python 3.9+ and pip or uv]

=== PHP / Slim

#link("4-5-php-integration.html")[Go to PHP Guide →]

\
#note[Prerequisites: PHP 8.3+ and Composer installed on your machine]

== Next Steps

Choose one of the integration guides above to continue. Each guide will show you how to:

1. Set up a basic web service in your chosen language/framework
2. Load and compile your Oicana template
3. Pass dynamic input values from your application code
4. Serve the generated PDFs to users

\
After completing one integration guide, you'll have a working service that can generate PDFs on demand!
