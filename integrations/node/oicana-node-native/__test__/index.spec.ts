import test from 'ava'
import fs from 'node:fs'

import {
  registerTemplate,
  compileTemplate,
  compileTemplateAsync,
  CompilationMode,
  exportDocument,
  exportDocumentAsync,
  removeDocument,
} from '../index.js'

const assetsDir = '../../../assets'

test('Can register and render template', async (t) => {
  console.time('template registration')
  const file = fs.readFileSync(`${assetsDir}/templates/invoice-0.1.0.zip`)
  const documentId = registerTemplate(
    'invoice',
    file,
    {
      invoice: `
  {
    "$schema": "invoice.schema.json",
  "id": "2024-03-10t172205",
  "issuingDate": "2024-04-27",
  "deliveryDate": "2024-04-19",
  "dueDate": "2024-05-06",
  "biller": {
    "name": "Gyro Gearloose",
    "title": "Inventor",
    "company": "dsadsa Inventions Ltd.",
    "vat-id": "DL1234567",
    "iban": "DE89370400440532013000",
    "address": {
      "country": "Disneyland",
      "city": "Duckburg",
      "postal-code": "123456",
      "street": "Inventor Drive 23"
    }
  },
  "recipient": {
    "name": "Scrooge McDuck",
    "title": "Treasure Hunter",
    "vat-id": "DL7654321",
    "address": {
      "country": "Disneyland",
      "city": "Duckburg",
      "postal-code": "123456",
      "street": "Killmotor Hill 1"
    }
  },
  "items": [
    {
      "date": "2016-04-03",
      "description": "Arc reactor",
      "quantity": 1,
      "price": 130
    },
    {
      "date": "2016-04-05",
      "description": "Flux capacitor",
      "quantity": 1,
      "price": 2700
    }
    ]
}`,
    },
    { banner: { bytes: fs.readFileSync(`${assetsDir}/logo/oicana_full_background_1024.png`), meta: '{}' } },
    CompilationMode.Development,
  )
  console.timeEnd('template registration')
  console.time('document export')
  const result = exportDocument(documentId, JSON.stringify({ format: 'pdf' }))
  console.timeEnd('document export')
  fs.writeFileSync('test.pdf', result)
  t.truthy(fs.existsSync('test.pdf'))

  console.time('template render')
  const secondDocumentId = compileTemplate(
    'invoice',
    {
      invoice: `
  {
    "$schema": "invoice.schema.json",
  "id": "2024-03-10t172205",
  "issuingDate": "2024-04-27",
  "deliveryDate": "2024-04-19",
  "dueDate": "2024-05-06",
  "biller": {
    "name": "Gyro Gearloose",
    "title": "Inventor",
    "company": "dsadsa Inventions Ltd.",
    "vat-id": "DL1234567",
    "iban": "DE89370400440532013000",
    "address": {
      "country": "Disneyland",
      "city": "Duckburg",
      "postal-code": "123456",
      "street": "Inventor Drive 23"
    }
  },
  "recipient": {
    "name": "Nikl McDuck",
    "title": "Treasure Hunter",
    "vat-id": "DL7654321",
    "address": {
      "country": "Germany",
      "city": "Duckburg",
      "postal-code": "123456",
      "street": "Killmotor Hill 1"
    }
  },
  "items": [
    {
      "date": "2021-04-03",
      "description": "CHANGED",
      "quantity": 42,
      "price": 99
    },
    {
      "date": "2016-04-05",
      "description": "Flux capacitor",
      "quantity": 1,
      "price": 2700
    },
    {
      "date": "2016-04-05",
      "description": "A new item",
      "quantity": 1,
      "price": 2700
    }
    ]
}`,
    },
    {
      banner: {
        bytes: fs.readFileSync(`${assetsDir}/logo/oicana_full_background_1024.png`),
        meta: JSON.stringify({ image_format: 'png' }),
      },
    },
    CompilationMode.Development,
  )
  console.timeEnd('template render')
  console.time('document export')
  const secondResult = exportDocument(secondDocumentId, JSON.stringify({ format: 'pdf' }))
  console.timeEnd('document export')
  fs.writeFileSync('test_2.pdf', secondResult)
  t.truthy(fs.existsSync('test_2.pdf'))
})

test('async compile and export keep the event loop free', async (t) => {
  const file = fs.readFileSync(`${assetsDir}/templates/invoice-0.1.0.zip`)
  const invoice = `
  {
    "$schema": "invoice.schema.json",
  "id": "2026-07-06t120000",
  "issuingDate": "2026-07-06",
  "deliveryDate": "2026-07-01",
  "dueDate": "2026-08-06",
  "biller": {
    "name": "Gyro Gearloose",
    "title": "Inventor",
    "company": "dsadsa Inventions Ltd.",
    "vat-id": "DL1234567",
    "iban": "DE89370400440532013000",
    "address": {
      "country": "Disneyland",
      "city": "Duckburg",
      "postal-code": "123456",
      "street": "Inventor Drive 23"
    }
  },
  "recipient": {
    "name": "Scrooge McDuck",
    "title": "Treasure Hunter",
    "vat-id": "DL7654321",
    "address": {
      "country": "Disneyland",
      "city": "Duckburg",
      "postal-code": "123456",
      "street": "Killmotor Hill 1"
    }
  },
  "items": [
    {
      "date": "2026-07-01",
      "description": "Async task",
      "quantity": 1,
      "price": 130
    }
    ]
}`
  const banner = { bytes: fs.readFileSync(`${assetsDir}/logo/oicana_full_background_1024.png`), meta: '{}' }
  const warmUpDocumentId = registerTemplate('invoice-async', file, { invoice }, { banner }, CompilationMode.Development)
  removeDocument(warmUpDocumentId)

  // If the native call blocked the event loop, its promise would already be
  // settled by the time the freshly scheduled setImmediate can run, and the
  // race would resolve to 'compile finished'.
  const compilePromise = compileTemplateAsync('invoice-async', { invoice }, { banner }, CompilationMode.Development)
  const compileWinner = await Promise.race([
    compilePromise.then(() => 'compile finished'),
    new Promise((resolve) => setImmediate(() => resolve('event loop turned'))),
  ])
  t.is(compileWinner, 'event loop turned')

  const documentId = await compilePromise
  const exportPromise = exportDocumentAsync(documentId, JSON.stringify({ format: 'pdf' }))
  const exportWinner = await Promise.race([
    exportPromise.then(() => 'export finished'),
    new Promise((resolve) => setImmediate(() => resolve('event loop turned'))),
  ])
  t.is(exportWinner, 'event loop turned')

  const result = await exportPromise
  t.is(result.subarray(0, 4).toString(), '%PDF')
  removeDocument(documentId)
})

test('async compile rejects for unknown templates', async (t) => {
  await t.throwsAsync(compileTemplateAsync('never-registered', {}, {}, CompilationMode.Development), {
    message: /not registered/i,
  })
})
