// Set PG_CON to run these. Example:
//   PG_CON=postgres://user:password@localhost:5432/pg yarn test
//
// cala-ledger runs its own migrations on connect, so any blank Postgres works.

import test from 'ava'

import { CalaLedger, ParamDataTypeValues } from '../index.js'

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

dbTest('posts a transaction whose template uses a CEL default param', async (t) => {
  // Regression: NewParamDefinition.default was silently dropped by the
  // binding, so any template that relied on a CEL default for required
  // fields (e.g. `effective: 'date()'`) would fail to post with
  // "Could not coerce Null into Date".
  const journal = await ledger.journals().create({ name: `tx-${Date.now()}` })
  const accounts = ledger.accounts()
  const sender = await accounts.create({
    code: `S-${Date.now()}-${Math.random()}`,
    name: 'sender',
  })
  const recipient = await accounts.create({
    code: `R-${Date.now()}-${Math.random()}`,
    name: 'recipient',
  })

  const code = `T_${Date.now()}_${Math.floor(Math.random() * 1e6)}`
  await ledger.txTemplates().create({
    code,
    params: [
      { name: 'sender', type: ParamDataTypeValues.Uuid },
      { name: 'recipient', type: ParamDataTypeValues.Uuid },
      { name: 'journal_id', type: ParamDataTypeValues.Uuid },
      { name: 'effective', type: ParamDataTypeValues.Date, default: 'date()' },
    ],
    transaction: {
      effective: 'params.effective',
      journalId: 'params.journal_id',
    },
    entries: [
      {
        entryType: "'DR'",
        accountId: 'params.sender',
        layer: "'SETTLED'",
        direction: "'DEBIT'",
        units: "decimal('1')",
        currency: "'USD'",
      },
      {
        entryType: "'CR'",
        accountId: 'params.recipient',
        layer: "'SETTLED'",
        direction: "'CREDIT'",
        units: "decimal('1')",
        currency: "'USD'",
      },
    ],
  })

  const tx = await ledger.transactions().post(code, {
    sender: sender.id(),
    recipient: recipient.id(),
    journal_id: journal.id(),
  })

  t.is(tx.values().journalId, journal.id())
  t.is(tx.values().entryIds.length, 2)
})
