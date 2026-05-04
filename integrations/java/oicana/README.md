# Oicana
*Dynamic PDF Generation based on Typst*

https://oicana.com

Oicana offers seamless PDF templating across multiple platforms. Define your templates in Typst, specify dynamic inputs, and generate high quality PDFs from any environment - whether it's a web browser, server application, or desktop software.

This artifact provides JNI bindings for compiling Oicana templates from Java applications.

## Installation

Add the API JAR plus the native JAR(s) for the platform(s) you target. With Gradle:

```kotlin
dependencies {
    implementation("com.oicana:oicana:<version>")
    runtimeOnly("com.oicana:oicana-linux-x86_64:<version>")
    runtimeOnly("com.oicana:oicana-linux-aarch64:<version>")
    runtimeOnly("com.oicana:oicana-macos-x86_64:<version>")
    runtimeOnly("com.oicana:oicana-macos-aarch64:<version>")
    runtimeOnly("com.oicana:oicana-windows-x86_64:<version>")
}
```

## Quick start

```java
import com.oicana.CompilationMode;
import com.oicana.ExportFormat;
import com.oicana.Template;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

byte[] templateBytes = Files.readAllBytes(Path.of("template.zip"));

try (var template = new Template(templateBytes)) {
    byte[] pdf = template.compile(
            Map.of("invoice", "{\"number\":\"INV-001\"}"),
            Map.of(),
            ExportFormat.pdf(),
            CompilationMode.PRODUCTION
    );
    Files.write(Path.of("output.pdf"), pdf);
}
```

## Open source example

You can find an example Spring Boot application using this artifact on GitHub: https://github.com/oicana/oicana-example-java-spring-boot

## Licensing

Oicana is source-available under [PolyForm Noncommercial License 1.0.0](./LICENSE.md). You can use it for free in any noncommercial context.
For commercial use, please visit [the Oicana website][oicana-website] for pricing options.


[oicana-website]: https://oicana.com
