use cala_ledger::primitives::DebitOrCredit;

#[napi(object)]
pub struct AccountBalanceValues {
  pub journal_id: String,
  pub account_id: String,
  pub currency: String,
  /// "DEBIT" or "CREDIT" — the account's normal side. Sign of the
  /// settled/pending/encumbrance amounts is interpreted against this.
  pub normal_side: String,
  pub settled: String,
  pub pending: String,
  pub encumbrance: String,
}

fn direction_str(d: DebitOrCredit) -> &'static str {
  match d {
    DebitOrCredit::Debit => "DEBIT",
    DebitOrCredit::Credit => "CREDIT",
  }
}

impl From<&cala_ledger::balance::AccountBalance> for AccountBalanceValues {
  fn from(b: &cala_ledger::balance::AccountBalance) -> Self {
    let details = &b.details;
    Self {
      journal_id: details.journal_id.to_string(),
      account_id: details.account_id.to_string(),
      currency: details.currency.to_string(),
      normal_side: direction_str(b.balance_type).to_string(),
      settled: b.settled().to_string(),
      pending: b.pending().to_string(),
      encumbrance: b.encumbrance().to_string(),
    }
  }
}

impl From<cala_ledger::balance::AccountBalance> for AccountBalanceValues {
  fn from(b: cala_ledger::balance::AccountBalance) -> Self {
    Self::from(&b)
  }
}
