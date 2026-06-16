mod values;

use std::str::FromStr;

use cala_ledger::primitives::{AccountId, Currency, JournalId};

pub use values::AccountBalanceValues;

#[napi]
pub struct CalaBalances {
  inner: cala_ledger::balance::Balances,
}

#[napi]
impl CalaBalances {
  pub fn new(inner: &cala_ledger::balance::Balances) -> Self {
    Self {
      inner: inner.clone(),
    }
  }

  /// Look up the current balance for `(journal, account, currency)`.
  /// Returns null when the account has no entries against this journal
  /// in the given currency yet (cala returns a NotFound error in that
  /// case; for a UI surface we'd rather show zero than throw).
  #[napi(ts_return_type = "Promise<AccountBalanceValues | null>")]
  pub async fn find(
    &self,
    journal_id: String,
    account_id: String,
    currency: String,
  ) -> napi::Result<Option<AccountBalanceValues>> {
    let journal_id = uuid::Uuid::parse_str(&journal_id)
      .map(JournalId::from)
      .map_err(crate::generic_napi_error)?;
    let account_id = uuid::Uuid::parse_str(&account_id)
      .map(AccountId::from)
      .map_err(crate::generic_napi_error)?;
    let currency = Currency::from_str(&currency).map_err(crate::generic_napi_error)?;

    match self.inner.find(journal_id, account_id, currency).await {
      Ok(balance) => Ok(Some(AccountBalanceValues::from(&balance))),
      Err(cala_ledger::balance::error::BalanceError::NotFound(_, _, _)) => Ok(None),
      Err(e) => Err(crate::generic_napi_error(e)),
    }
  }
}
