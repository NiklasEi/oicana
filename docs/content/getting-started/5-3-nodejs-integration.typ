#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *

In this chapter, you'll integrate Oicana into a Node.js web service using #link("https://nestjs.com/")[NestJS]. NestJS is a progressive Node.js framework for building efficient, scalable server-side applications. It uses TypeScript by default and provides a modular architecture. We'll create a simple web service that compiles your Oicana template to PDF and serves it via an HTTP endpoint.

\
#note[This chapter assumes that you have a working Node.js 18+ setup with npm. If that is not the case, please follow #link("https://nodejs.org/en/download/")[the official Node.js installation guide] to install Node.js on your machine.]

\
Let's start with a fresh NestJS project by executing `npx @nestjs/cli new oicana-demo` in a new directory. This will create a new NestJS application with a basic structure. The starter project has a single endpoint defined in the controller. We can test it by starting the service with `npm run start:dev` and navigating to `http://localhost:3000` in a browser.

== New service endpoint

We will define a new endpoint to compile our Oicana template to a PDF and return the PDF file to the user.

\
1. Create a new directory in the Node.js project called `templates` and copy `example-0.1.0.zip` into that directory.
2. Add the #link("https://www.npmjs.com/package/@oicana/node")[`@oicana/node` npm package] as a dependency with `npm install @oicana/node`.
3. Generate a new controller and service for templates:
  ```bash
  npx nest generate module templates
  npx nest generate service templates
  npx nest generate controller templates
  ```

4. Update the templates service to load the template at startup:

  \
  #code("src/templates/templates.service.ts", ```typescript
  import { Injectable, OnModuleInit } from '@nestjs/common';
  import { Template, CompilationMode } from '@oicana/node';
  import { promises as fs } from 'fs';
  import { join } from 'path';

  @Injectable()
  export class TemplatesService implements OnModuleInit {
    private template: Template;

    async onModuleInit() {
      const templatePath = join(
        process.cwd(),
        'templates',
        'example-0.1.0.zip'
      );
      const buffer = await fs.readFile(templatePath);
      // Template registration defaults to Development mode
      // so it will use the development value of our template input
      this.template = new Template('example', buffer);
    }

    compile(): Uint8Array {
      const jsonInputs = new Map();
      const blobInputs = new Map();
      return this.template.compile(
        jsonInputs,
        blobInputs,
        { format: 'pdf' },
        CompilationMode.Development
      );
    }
  }
  ```)

  \
5. Update the templates controller to add a compile endpoint:

  #code(
    "src/templates/templates.controller.ts",
    ```typescript
    import { Controller, Post, Res } from '@nestjs/common';
    import type { Response } from 'express';
    import { TemplatesService } from './templates.service';

    @Controller('templates')
    export class TemplatesController {
      constructor(
        private readonly templatesService: TemplatesService
      ) {}

      @Post('compile')
      compile(@Res() res: Response) {
        const pdf = this.templatesService.compile();

        res.set({
          'Content-Type': 'application/pdf',
          'Content-Disposition': 'attachment; filename="example.pdf"',
          'Content-Length': pdf.length,
        });
        res.status(200).end(Buffer.from(pdf));
      }
    }
    ```,
  )

  This code defines a new POST endpoint at `/templates/compile`. For every request, it compiles the template with empty input maps and returns the PDF file. We explicitly pass `CompilationMode.Development` here to so the template uses the development value you defined for the `info` input ("Chuck Norris"). In a follow up step, we will set an input value instead.

After restarting the service, you can test the endpoint with curl:

```bash
curl -X POST http://localhost:3000/templates/compile --output example.pdf
```

The generated `example.pdf` file should contain your template with the development value.

== About performance

The PDF generation should not take longer than a couple of milliseconds.

\
For better performance in production environments with heavy load, consider moving compilation to worker threads. This allows you to offload CPU-intensive compilation work from the main event loop. Libraries like #link("https://github.com/piscinajs/piscina")[piscina] can help with that.

== Passing inputs from Node.js

Our `compile` method is currently calling `template.compile()` with empty input maps and development mode. Now we'll provide explicit input values and switch to production mode:

#code(
  "Part of src/templates/templates.service.ts",
  ```typescript
  compile(): Uint8Array {
    const jsonInputs = new Map<string, string>();
    const blobInputs = new Map();

    jsonInputs.set('info', JSON.stringify({ name: 'Baby Yoda' }));

    return this.template.compile(jsonInputs, blobInputs);
  }
  ```,
)

\
Notice that we removed the explicit `CompilationMode.Development` parameter. The `compile()` method defaults to `CompilationMode.Production` when no mode is specified. Production mode is the recommended default for all document compilation in your application - it ensures you never accidentally generate a document with test data. In production mode, the template will never fall back to development values. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles `none` values for that input.

\
Calling the endpoint now will result in a PDF with "Baby Yoda" instead of "Chuck Norris". Building on this minimal service, you could set input values based on database entries or the request payload. Take a look at the #link("https://github.com/oicana/oicana-example-nestjs/")[open source NestJS example project on GitHub] for a more complete showcase of the Oicana Node.js integration, including blob inputs, error handling, and Swagger documentation.
