#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *

In this chapter, you'll integrate Oicana into a Java web service using #link("https://spring.io/projects/spring-boot")[Spring Boot]. Spring Boot is a popular Java framework for building production-ready web services. We'll create a simple web service that compiles your Oicana template to PDF and serves it via an HTTP endpoint.

\
#note[This chapter assumes that you have a working Java 17+ setup with Gradle. If that is not the case, please follow #link("https://adoptium.net/")[the Adoptium installation guide] to install a JDK on your machine.]

\
Let's start with a fresh Spring Boot project. The quickest way is to use #link("https://start.spring.io/")[Spring Initializr]. Generate a project with "Spring Web" as the only dependency and extract it into a new directory.

Alternatively, create a project manually:

```bash
mkdir my-pdf-service
cd my-pdf-service
```

Initialize it with the following `build.gradle.kts`:

\
#code("build.gradle.kts", ```kotlin
plugins {
    java
    id("org.springframework.boot") version "3.4.3"
    id("io.spring.dependency-management") version "1.1.7"
}

group = "com.example"
version = "1.0.0"

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.springframework.boot:spring-boot-starter-web")
    implementation("com.oicana:oicana:0.1.0-alpha.1")
    // Add the native library for your platform.
    // You can add multiple if your team uses different platforms.
    runtimeOnly("com.oicana:oicana-linux-x86_64:0.1.0-alpha.1")
}
```)

\
#note[Replace the `runtimeOnly` dependency with the native library for your platform. Available options: `oicana-linux-x86_64`, `oicana-linux-aarch64`, `oicana-macos-x86_64`, `oicana-macos-aarch64`, `oicana-windows-x86_64`. You can add multiple if your team uses different platforms - only the matching one will be loaded at runtime.]

Create the main application class:

\
#code("src/main/java/com/example/Application.java", ```java
package com.example;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```)

You can test it by running ```bash ./gradlew bootRun``` and navigating to #link("http://localhost:8080") in your browser.

== New service endpoint

We will define a new endpoint to compile our Oicana template to a PDF and return the PDF file to the user.

\
1. Create a new directory in the project called `templates` and copy `example-0.1.0.zip` into that directory.
2. Create a service to load and compile the template:

  \
  #code("src/main/java/com/example/TemplateService.java", ```java
  package com.example;

  import com.oicana.CompilationMode;
  import com.oicana.ExportFormat;
  import com.oicana.Template;
  import jakarta.annotation.PostConstruct;
  import jakarta.annotation.PreDestroy;
  import org.springframework.stereotype.Service;

  import java.io.IOException;
  import java.nio.file.Files;
  import java.nio.file.Path;
  import java.util.Map;

  @Service
  public class TemplateService {
      private Template template;

      @PostConstruct
      public void init() throws IOException {
          byte[] templateBytes = Files.readAllBytes(
              Path.of("templates/example-0.1.0.zip")
          );
          template = new Template(templateBytes);
      }

      public byte[] compile() {
          return template.compile(
              Map.of(),
              Map.of(),
              ExportFormat.pdf(),
              CompilationMode.DEVELOPMENT
          );
      }

      @PreDestroy
      public void cleanup() {
          template.close();
      }
  }
  ```)
  The `Template` constructor loads the template once. The `compile` method compiles it with empty inputs and ```java CompilationMode.DEVELOPMENT``` so the template uses the development value you defined for the `info` input (```json { "name": "Chuck Norris" }```). In a follow-up step, we will set an input value instead. The `@PreDestroy` cleanup releases native resources.

  \
3. Create a controller with a compile endpoint:

  #code("src/main/java/com/example/CompileController.java", ```java
  package com.example;

  import org.springframework.http.HttpHeaders;
  import org.springframework.http.MediaType;
  import org.springframework.http.ResponseEntity;
  import org.springframework.web.bind.annotation.PostMapping;
  import org.springframework.web.bind.annotation.RestController;

  @RestController
  public class CompileController {
      private final TemplateService templateService;

      public CompileController(TemplateService templateService) {
          this.templateService = templateService;
      }

      @PostMapping("/compile")
      public ResponseEntity<byte[]> compile() {
          byte[] pdf = templateService.compile();

          return ResponseEntity.ok()
              .header(HttpHeaders.CONTENT_DISPOSITION,
                  "attachment; filename=\"example.pdf\"")
              .contentType(MediaType.APPLICATION_PDF)
              .body(pdf);
      }
  }
  ```)

  This code defines a new POST endpoint at `/compile`. For every request, it compiles the template and returns the PDF file.

After restarting the service, you can test the endpoint with curl:

```bash
curl -X POST http://localhost:8080/compile --output example.pdf
```

The generated `example.pdf` file should contain your template with the development value.

== About performance

The PDF generation should not take longer than a couple of milliseconds. The `Template` instance is thread-safe and can be shared across requests - Spring Boot's singleton service scope handles this naturally.

== Passing inputs from Java

Our `compile` method is currently calling ```java template.compile()``` with empty inputs and development mode. Now we'll provide an explicit input value and switch to production mode:

#code(
  "Part of TemplateService.java",
  ```java
  public byte[] compile() {
      return template.compile(
          Map.of("info", "{\"name\": \"Baby Yoda\"}")
      );
  }
  ```,
)

\
Notice that we switched to the simpler ```java compile(Map<String, String>)``` overload which defaults to ```java CompilationMode.PRODUCTION``` and PDF output. Production mode is the recommended default for all document compilation in your application - it ensures you never accidentally generate a document with test data. In production mode, the template will never fall back to development values for inputs. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles ```typst none``` values for that input.

\
Calling the endpoint now will result in a PDF with "Baby Yoda" instead of "Chuck Norris". Building on this minimal service, you could set input values based on database entries or the request payload. Take a look at the #link("https://github.com/oicana/oicana-example-java-spring-boot/")[open source Spring Boot example project on GitHub] for a more complete showcase of the Oicana Java integration.
