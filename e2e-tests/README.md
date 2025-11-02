# E2E Tests for Oicana Example Services

This directory contains end-to-end tests for the Oicana example services using Bruno.

## Overview

The Bruno test collection validates that all example services can successfully compile all example templates. This ensures that end users can clone and run the example services with confidence.


## Templates Tested

The tests validate all 8 example templates:

1. **minimal** - Basic template with no inputs
2. **certificate** - Certificate generation template
3. **table** - Table rendering with JSON input
4. **fonts** - Font showcase template
5. **dependency** - Template with Typst package dependencies
6. **invoice** - Invoice generation with structured data
7. **invoice_zugferd** - ZUGFeRD-compliant invoice (PDF/A-3b)
8. **multi_input** - Template demonstrating multiple JSON and blob inputs

Each template is tested with both:
- **compile** endpoint (generates PDF)
- **preview** endpoint (generates PNG)

## Prerequisites

### Bruno CLI

Install Bruno CLI globally:

```bash
npm install -g @usebruno/cli
```

Or use npx without installation:

```bash
npx @usebruno/cli run bruno/
```

### Running Services

The tests expect services to be running on these ports:

- **ASP.NET**: http://localhost:3002
- **Axum**: http://localhost:3000
- **NestJS**: http://localhost:3001

Start each service before running the tests.

## Running Tests

### Test All Services

Run tests against all services:

```bash
./run-bruno-tests.sh
```

### Test Specific Service

Run tests for a single service:

```bash
# Test only Axum
./run-bruno-tests.sh axum

# Test ASP.NET and NestJS
./run-bruno-tests.sh aspnet,nestjs
```

### Using Bruno CLI Directly

Run tests manually:

```bash
# Test Axum service
bru run bruno/ --env axum

# Test ASP.NET service
bru run bruno/ --env aspnet

# Test NestJS service
bru run bruno/ --env nestjs
```

### Generate HTML Report

```bash
bru run bruno/ --env axum --reporter-html results.html
```

## Using Bruno GUI

You can also open the collection in Bruno Desktop for interactive testing:

1. Download Bruno from https://www.usebruno.com/
2. Open Bruno
3. Click "Open Collection"
4. Select the `e2e-tests/bruno` directory
5. Choose an environment (aspnet, axum, or nestjs)
6. Run individual tests or the entire collection

## Test Coverage

### API Endpoints

- ✅ `GET /templates` - List all available templates
- ✅ `POST /templates/{id}/compile` - Generate PDF for each template
- ✅ `POST /templates/{id}/preview` - Generate PNG preview for each template

### Validations

Each test validates:
- HTTP status code (200 OK)
- Response content type (application/pdf or image/png)
- Response body is not empty

## CI/CD Integration

The tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [aspnet, axum, nestjs]

    steps:
      - uses: actions/checkout@v3

      - name: Install Bruno CLI
        run: npm install -g @usebruno/cli

      - name: Start ${{ matrix.service }} service
        run: # ... start the service

      - name: Run Bruno tests
        run: bru run e2e-tests/bruno --env ${{ matrix.service }}
```

## Extending Tests

### Adding New Template Tests

1. Create two new `.bru` files in `bruno/templates/`:
   - `{template}-compile.bru` - For PDF generation
   - `{template}-preview.bru` - For PNG preview

2. Use the existing templates as reference

3. Adjust the `body:json` section with appropriate inputs for your template

### Adding New Services

1. Create a new environment file: `bruno/environments/{service}.bru`

2. Set the correct `baseUrl` and `serviceName`

3. Run tests: `./run-bruno-tests.sh {service}`

## Troubleshooting

### Service Not Running

```
Error: connect ECONNREFUSED 127.0.0.1:3000
```

**Solution**: Make sure the service is running on the expected port.

### Template Not Found (404)

**Solution**: Ensure the template `.zip` files are present in the service's `templates/` directory.

### Compilation Errors (400)

**Solution**: Check that the test inputs match the template's expected schema. Review the template's `typst.toml` for input definitions.

## Future Enhancements

- [ ] Snapshot testing - Compare generated PDFs/PNGs across services
- [ ] Performance benchmarks - Track compilation times
- [ ] Blob upload tests - Test the `/blobs` endpoint
- [ ] Error handling tests - Validate error responses
- [ ] Template download tests - Test `GET /templates/{id}`
