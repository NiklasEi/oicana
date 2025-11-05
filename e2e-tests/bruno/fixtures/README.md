# Test Fixtures

This directory contains shared JSON test data that is loaded dynamically by Bruno test files.

## Available Fixtures

- **`invoice.json`** - Complete invoice data for testing both `invoice` and `invoice_zugferd` templates
- **`table-data.json`** - Table data with rows for testing the `table` template
- **`multi-input-one.json`** - First JSON input for `multi_input` template
- **`multi-input-two.json`** - Second JSON input for `multi_input` template

## How It Works

Bruno supports loading external JSON files via filesystem access. This is enabled in `bruno.json`:

```json
{
  "scripts": {
    "filesystemAccess": {
      "allow": true
    }
  }
}
```

## Usage in Bruno Tests

Tests use pre-request scripts to load fixture files:

```javascript
script:pre-request {
  const fs = require('fs');
  const path = require('path');

  // Load fixture from file
  // Bruno CLI runs from collection root, so use process.cwd()
  const fixturePath = path.join(process.cwd(), 'fixtures', 'invoice.json');
  const invoiceData = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));

  // Set the request body
  req.setBody({
    jsonInputs: [
      {
        key: "invoice",
        value: invoiceData
      }
    ],
    blobInputs: []
  });
}
```

## Benefits

1. **Single source of truth** - Update invoice data in one place
2. **No duplication** - Both compile and preview tests share the same fixture
3. **Maintainability** - Update fixture file once, all tests automatically use new data
4. **Consistency** - Ensures all services are tested with identical input data
5. **Version control friendly** - Changes to test data are clearly visible in git diffs

## Test File Mapping

Each fixture file is used by multiple test files:

- **`invoice.json`** → Used by:
  - `invoice-compile.bru`
  - `invoice-preview.bru`
  - `invoice_zugferd-compile.bru`
  - `invoice_zugferd-preview.bru`

- **`table-data.json`** → Used by:
  - `table-compile.bru`
  - `table-preview.bru`

- **`multi-input-one.json` & `multi-input-two.json`** → Used by:
  - `multi_input-compile.bru`
  - `multi_input-preview.bru`

## Updating Test Data

To update test data:

1. Edit the appropriate fixture JSON file
2. Commit the change
3. Run tests - they will automatically use the new data

No need to update individual `.bru` files!
