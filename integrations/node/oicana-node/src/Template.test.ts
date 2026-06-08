import { readFile, writeFile } from 'node:fs/promises';
import { describe, expect, it } from 'vitest';
import { CompilationMode } from './CompilationMode';
import { Png } from './ExportFormat';
import type { BlobWithMetadata } from './inputs';
import { Template } from './Template';

const asset = (file: string) => {
  return readFile(`../../../assets/${file}`);
};

describe('e2e test template', () => {
  it('development', async () => {
    const templateFile = await readFile(
      '../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip',
    );
    const template = new Template(templateFile);

    const image = template.export(
      new Map(),
      new Map(),
      Png(1),
      CompilationMode.Development,
    );

    await writeFile('testOutput/development.png', image);
  });

  it('production', async () => {
    const templateFile = await readFile(
      '../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip',
    );
    const template = new Template(templateFile);

    const blob = await asset('inputs/input.txt');
    const json = await asset('inputs/input.json');

    const blobInputs = new Map<string, BlobWithMetadata>();
    blobInputs.set('development-blob', {
      bytes: blob,
      meta: { image_format: 'jpeg', foo: 43, bar: ['input', 'two'] },
    });
    const jsonInputs = new Map<string, string>();
    jsonInputs.set('development-json', json.toString());

    const image = template.export(jsonInputs, blobInputs, Png(1));

    await writeFile('testOutput/production.png', image);
  });

  it('all-inputs', async () => {
    const templateFile = await readFile(
      '../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip',
    );
    const template = new Template(templateFile);

    const blob = await asset('inputs/input.txt');
    const json = await asset('inputs/input.json');

    const blobInputs = new Map<string, BlobWithMetadata>();
    blobInputs.set('default-blob', {
      bytes: blob,
      meta: { image_format: 'jpeg', foo: 42, bar: ['input', 'two'] },
    });
    blobInputs.set('development-blob', {
      bytes: blob,
      meta: { image_format: 'jpeg', foo: 43, bar: ['input', 'two'] },
    });
    blobInputs.set('both-blob', {
      bytes: blob,
      meta: { image_format: 'jpeg', foo: 44, bar: ['input', 'two'] },
    });
    const jsonInputs = new Map<string, string>();
    jsonInputs.set('default-json', json.toString());
    jsonInputs.set('development-json', json.toString());
    jsonInputs.set('both-json', json.toString());

    const image = template.export(jsonInputs, blobInputs, Png(1));

    await writeFile('testOutput/all-inputs.png', image);
  });

  it('explicit development mode allows compile with empty inputs', async () => {
    const templateFile = await readFile(
      '../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip',
    );
    const template = new Template(templateFile);

    template.export(new Map(), new Map(), Png(1), CompilationMode.Development);
  });

  it('export defaults to production mode', async () => {
    const templateFile = await readFile(
      '../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip',
    );
    const template = new Template(templateFile);

    expect(() => {
      template.export(new Map(), new Map(), Png(1));
    }).toThrow(/No value for the required input/);
  });

  it('can control compilation mode when registering', async () => {
    const templateFile = await readFile(
      '../../../e2e-tests/template/oicana-e2e-test-x.y.z.zip',
    );
    expect(() => {
      new Template(
        templateFile,
        new Map(),
        new Map(),
        CompilationMode.Production,
      );
    }).toThrow(/No value for the required input/);
  });
});
