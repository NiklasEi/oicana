import test from 'ava'
import fs from 'node:fs'

import { registerTemplate, compileTemplate, ExportFormat } from '../index.js'

const assetsDir = '../../../assets'

test('Can register and render template', async (t) => {
  console.time('template registration')
  const file = fs.readFileSync(`${assetsDir}/templates/test-0.1.0.zip`)
  const result = registerTemplate(
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
    { banner: { bytes: fs.readFileSync(`${assetsDir}/logo/oicana_full_background_1024.png`), meta: {} } },
    ExportFormat.Pdf,
  )
  console.timeEnd('template registration')
  fs.writeFileSync('test.pdf', result)
  t.truthy(fs.existsSync('test.pdf'))

  console.time('template render')
  const secondResult = compileTemplate(
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
        meta: { imageFormat: 'png' },
      },
    },
    ExportFormat.Pdf,
  )
  console.timeEnd('template render')
  fs.writeFileSync('test_2.pdf', secondResult)
  t.truthy(fs.existsSync('test_2.pdf'))
})
