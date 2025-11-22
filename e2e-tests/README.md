# E2E Tests for Oicana Example Services

This directory contains end-to-end test related files.

`template` is an Oicana template used for e2e testing integrations via snapshot tests
`bruno` contains bruno tests for the Oicana example services

## Bruno

The Bruno test collection validates that all example services can successfully compile all example templates. This ensures that end users can clone and run the example services with confidence.

The test collection is executed against the example services in CI pieplines in the service repositories.

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
