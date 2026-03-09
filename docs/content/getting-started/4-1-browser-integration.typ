#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *

In this chapter, you'll integrate Oicana into a React application using WebAssembly. Unlike the server-side integrations, Oicana runs entirely in the browser here - no server required for PDF generation.

\
#note[This chapter assumes that you have a working Node.js 18+ setup with npm. If that is not the case, please follow #link("https://nodejs.org/en/download/")[the official Node.js installation guide] to install Node.js on your machine.]

\
Let's start with a fresh React project using Vite. Run the following command:

```bash
npm create vite@latest my-pdf-app -- --template react-ts
cd my-pdf-app
npm install
```

You can test it by running ```bash npm run dev``` and navigating to #link("http://localhost:5173") in your browser.

== Adding Oicana

Install the Oicana browser packages:

```bash
npm install @oicana/browser @oicana/browser-wasm
```

\
1. Create a `public` directory (if it doesn't exist) and copy `example-0.1.0.zip` into it. This makes the template accessible to the browser.

2. Replace the contents of `src/App.tsx` with the following:

  \
  #code("src/App.tsx", ```tsx
  import { useEffect, useState } from 'react';
  import { initialize, Template, CompilationMode, Pdf } from '@oicana/browser';
  import wasmUrl from '@oicana/browser-wasm/oicana_browser_wasm_bg.wasm?url';

  function App() {
    const [template, setTemplate] = useState<Template | null>(null);

    useEffect(() => {
      async function setup() {
        await initialize(wasmUrl);
        const response = await fetch('/example-0.1.0.zip');
        const templateBytes = new Uint8Array(await response.arrayBuffer());
        setTemplate(new Template(templateBytes));
      }
      void setup();
    }, []);

    function generatePdf() {
      const pdf = template!.compile(
        new Map(),
        new Map(),
        Pdf,
        CompilationMode.Development,
      );

      const blob = new Blob([pdf], { type: 'application/pdf' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'example.pdf';
      a.click();
      URL.revokeObjectURL(url);
    }

    if (!template) {
      return <p>Template is being prepared...</p>;
    }

    return (
      <button onClick={generatePdf}>
        Generate PDF
      </button>
    );
  }

  export default App;
  ```)

Start the dev server with ```bash npm run dev``` and click the "Generate PDF" button. The generated `example.pdf` file should contain your template with the development value.

== About performance

To keep the UI responsive, it's recommended to move template compilation to a #link("https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API")[Web Worker]. This prevents the main thread from being blocked during compilation. Take a look at the #link("https://github.com/oicana/oicana-example-typescript-react/")[open source React example project on GitHub] for a Web Worker setup.

== Passing inputs from TypeScript

Our code currently compiles with empty inputs and development mode. This means Oicana will use the default value for the input. Now we'll provide an explicit input value and switch to production mode. Update the `generatePdf` function:

#code(
  "Part of src/App.tsx",
  ```tsx
  function generatePdf() {
    const jsonInputs = new Map<string, string>();
    jsonInputs.set('info', JSON.stringify({ name: 'Baby Yoda' }));

    const pdf = template!.compile(
      jsonInputs,
      new Map(),
      Pdf(),
    );

    const blob = new Blob([pdf], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'example.pdf';
    a.click();
    URL.revokeObjectURL(url);
  }
  ```,
)

\
Notice that we removed the explicit ```ts CompilationMode.Development``` parameter. The ```ts compile()``` method defaults to ```ts CompilationMode.Production```. Production mode is the recommended default - it ensures you never accidentally generate a document with test data. In production mode, the template will never fall back to development values for inputs. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles ```typst none``` values for that input.

\
Clicking the button now will download a PDF with "Baby Yoda" instead of "Chuck Norris". For a more complete example including Web Workers, take a look at the #link("https://github.com/oicana/oicana-example-typescript-react/")[open source React example project on GitHub].
