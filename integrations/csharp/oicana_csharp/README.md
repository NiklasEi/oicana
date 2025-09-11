# Oicana csharp

This crate exposes FFI bindings for the C# wrapper package `../Oicana`.

## Development

```sh
cargo run --example csharp_bindings -p oicana_csharp && cargo build --release -p oicana_csharp
```

The command above generates the C# class and creates the dynamic library. The C# class will be written directly to the C# wrapper project.
The C# wrapper will copy the library build from the `target` directory to its own `bin` directory at build time (only in
Debug mode).

## .NET support

This library currently supports .NET 7 and higher. If you need support for older versions, please contact `info@oicana.com`.
