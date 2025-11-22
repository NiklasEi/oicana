# Oicana

Dynamic PDF Generation based on Typst.

Oicana offers seamless PDF templating. Define your templates in Typst, specify dynamic inputs, and generate high quality PDFs from your application code.

## Installation

```bash
pip install oicana
```

## Quick Start

```python
import json
from oicana import Template, CompilationMode

# Load your template
with open("template.zip", "rb") as f:
    template_bytes = f.read()

# Create template and compile
with Template(template_bytes) as template:
    pdf = template.compile(
        json_inputs={
            "invoice": json.dumps({
                "number": "INV-001",
                "items": [{"name": "Service", "price": 100}]
            })
        },
        export_format={"format": "pdf"},
        mode=CompilationMode.PRODUCTION,
    )

    # Save the PDF
    with open("output.pdf", "wb") as f:
        f.write(pdf)
```

## Features

- **Multi-platform**: Works on Linux, macOS, and Windows
- **Powerful Layouting**: Full access to Typst's functionality
- **Performant**: Native Rust performance via PyO3
- **Type Safe**: Full type hints for IDE support
- **Pythonic**: Context managers, dataclasses, and enums

## Documentation

For more information, visit [https://docs.oicana.com](https://docs.oicana.com)

## License

This project is licensed under the PolyForm Noncommercial License 1.0.0.
See the [LICENSE.md](LICENSE.md) file for details.
