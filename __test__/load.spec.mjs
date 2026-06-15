import test from 'ava'

import * as cala from '../index.js'

const EXPECTED_CLASSES = [
  'CalaLedger',
  'CalaAccounts',
  'CalaAccount',
  'CalaJournals',
  'CalaJournal',
  'CalaTransactions',
  'CalaTransaction',
  'CalaTxTemplates',
  'CalaTxTemplate',
]

test('native binding loads', (t) => {
  t.truthy(cala, 'expected the native module to load')
})

for (const name of EXPECTED_CLASSES) {
  test(`exports ${name} as a class`, (t) => {
    t.is(typeof cala[name], 'function', `${name} should be a constructor`)
  })
}

test('CalaLedger exposes static connect()', (t) => {
  t.is(typeof cala.CalaLedger.connect, 'function')
})

test('ParamDataTypeValues enum is exported', (t) => {
  t.is(typeof cala.ParamDataTypeValues, 'object')
  for (const variant of [
    'String',
    'Integer',
    'Decimal',
    'Boolean',
    'Uuid',
    'Date',
    'Timestamp',
    'Json',
  ]) {
    t.true(variant in cala.ParamDataTypeValues, `missing variant ${variant}`)
  }
})
