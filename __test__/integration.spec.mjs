// Set PG_CON to run these. Example:
//   PG_CON=postgres://user:password@localhost:5432/pg yarn test
//
// cala-ledger runs its own migrations on connect, so any blank Postgres works.

import test from 'ava'

import { CalaLedger } from '../index.js'

const PG_CON = process.env.PG_CON
const dbTest = PG_CON ? test : test.skip

let ledger

test.before(async () => {
  if (!PG_CON) return
  ledger = await CalaLedger.connect({ pgCon: PG_CON })
})

dbTest('connects to Postgres', (t) => {
  t.truthy(ledger)
  t.is(typeof ledger.accounts, 'function')
})

dbTest('creates and finds a journal', async (t) => {
  const journals = ledger.journals()
  const name = `smoke-${Date.now()}`

  const created = await journals.create({ name })
  const id = created.id()
  t.true(typeof id === 'string' && id.length > 0)
  t.is(created.values().name, name)

  const found = await journals.find(id)
  t.is(found.id(), id)
  t.is(found.values().name, name)
})

dbTest('lists accounts with pagination shape', async (t) => {
  const accounts = ledger.accounts()
  const result = await accounts.list({ first: 5 })

  t.true(Array.isArray(result.accounts))
  t.is(typeof result.hasNextPage, 'boolean')
  // endCursor is Option<CursorToken> — either undefined/null or { token: string }
  if (result.endCursor != null) {
    t.is(typeof result.endCursor.token, 'string')
  }
})
