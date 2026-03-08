#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *

In this chapter, you'll integrate Oicana into a Python web service using #link("https://fastapi.tiangolo.com/")[FastAPI]. FastAPI is a modern, high-performance web framework for building APIs with Python based on standard Python type hints. We'll create a simple web service that compiles your Oicana template to PDF and serves it via an HTTP endpoint.

\
#note[This chapter assumes that you have a working Python 3.9+ setup with pip or uv. If that is not the case, please follow #link("https://www.python.org/downloads/")[the official Python installation guide] to install Python on your machine.]

\
Let's start with a fresh FastAPI project. First, create a new directory for your project, then set up a virtual environment and install FastAPI with ```bash pip install fastapi uvicorn``` (or ```bash uv add fastapi uvicorn``` if using uv). Create a `main.py` file with the following basic FastAPI application:
\
#code("main.py", ```python
from fastapi import FastAPI

app = FastAPI()


@app.get("/")
async def root():
    return {"message": "Hello World"}
```)


You can test it by running ```bash uvicorn main:app --reload``` (or ```bash fastapi dev main.py``` if using FastAPI CLI) and navigating to #link("http://localhost:8000") in your browser.

== New service endpoint

We will define a new endpoint to compile our Oicana template to a PDF and return the PDF file to the user.

\
1. Create a new directory in the Python project called `templates` and copy `example-0.1.0.zip` into that directory.
2. Add the #link("https://pypi.org/project/oicana/")[`oicana` PyPI package] as a dependency with ```bash pip install oicana``` (or ```bash uv add oicana```).
3. Update `main.py` to load the template at startup and add a compile endpoint:

  \
  #code("main.py", ```python
  from contextlib import asynccontextmanager
  from pathlib import Path

  from fastapi import FastAPI
  from fastapi.responses import Response
  from oicana import Template, CompilationMode


  template: Template


  @asynccontextmanager
  async def lifespan(app: FastAPI):
      global template

      template_path = Path("templates/example-0.1.0.zip")
      template_bytes = template_path.read_bytes()
      # Template registration uses development mode by default
      template = Template(template_bytes)
      yield


  app = FastAPI(lifespan=lifespan)


  @app.post("/compile")
  async def compile_template():
      pdf = template.compile(mode=CompilationMode.DEVELOPMENT)

      return Response(
          content=bytes(pdf),
          media_type="application/pdf",
          headers={
              "Content-Disposition": "attachment; filename=example.pdf"
          },
      )
  ```)

  \
  This code loads the template once at application startup using FastAPI's lifespan context manager. The `/compile` endpoint compiles the template and returns the PDF file. We explicitly pass the compilation mode with ```python template.compile(mode=CompilationMode.DEVELOPMENT)``` so the template uses the development value you defined for the `info` input (```json { "name": "Chuck Norris" }```). In a follow-up step, we will set an input value instead.

After restarting the service, you can test the endpoint with curl:

```bash
curl -X POST http://localhost:8000/compile --output example.pdf
```

The generated `example.pdf` file should contain your template with the development value.

\
You can also explore the automatically generated API documentation by navigating to #link("http://localhost:8000/docs") in your browser. FastAPI provides interactive API documentation out of the box.

== About performance

The PDF generation should not take longer than a couple of milliseconds.

\
For better performance in production environments with heavy load, consider using FastAPI's support for async operations and deploying with multiple worker processes using Gunicorn or similar ASGI servers.

== Passing inputs from Python

Our ```python compile_template``` function is currently calling ```python template.compile()``` with development mode. Now we'll provide explicit input values and switch to production mode:

#code(
  "Part of main.py",
  ```python
  import json

  @app.post("/compile")
  async def compile_template():
      pdf = template.compile(
          json_inputs={"info": json.dumps({"name": "Baby Yoda"})}
      )

      return Response(
          content=bytes(pdf),
          media_type="application/pdf",
          headers={
              "Content-Disposition": "attachment; filename=example.pdf"
          },
      )
  ```,
)

\
Notice that we removed the explicit ```python mode=CompilationMode.DEVELOPMENT``` parameter. The ```python compile()``` method defaults to ```python CompilationMode.PRODUCTION``` when no mode is specified. Production mode is the recommended default for all document compilation in your application - it ensures you never accidentally generate a document with test data. In production mode, the template will never fall back to development values. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles ```typst none``` values for that input.

\
Calling the endpoint now will result in a PDF with "Baby Yoda" instead of "Chuck Norris". Building on this minimal service, you could set input values based on database entries or the request payload. Take a look at the #link("https://github.com/oicana/oicana-example-python-fastapi/")[open source FastAPI example project on GitHub] for a more complete showcase of the Oicana Python integration, including blob inputs, error handling, and request models.
