mod values;

use std::str::FromStr;

use cala_ledger::primitives::{AccountId, Currency, JournalId};
use rust_decimal::Decimal;

pub use values::{AccountBalanceValues, JournalTotalsValues};

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

  /// Sum the gross DR and CR sides of SETTLED balances across a set of
  /// accounts within one (journal, currency). Aggregation is done over
  /// cala-returned snapshot values — no arithmetic happens outside
  /// `Decimal`s cala produced. Accounts that have never been touched
  /// in this journal/currency simply don't contribute.
  #[napi]
  pub async fn journal_totals(
    &self,
    journal_id: String,
    account_ids: Vec<String>,
    currency: String,
  ) -> napi::Result<JournalTotalsValues> {
    let journal_id = uuid::Uuid::parse_str(&journal_id)
      .map(JournalId::from)
      .map_err(crate::generic_napi_error)?;
    let currency = Currency::from_str(&currency).map_err(crate::generic_napi_error)?;

    let ids: Vec<(JournalId, AccountId, Currency)> = account_ids
      .iter()
      .map(|s| {
        uuid::Uuid::parse_str(s)
          .map(|u| (journal_id, AccountId::from(u), currency))
          .map_err(crate::generic_napi_error)
      })
      .collect::<napi::Result<Vec<_>>>()?;

    let balances = self
      .inner
      .find_all(&ids)
      .await
      .map_err(crate::generic_napi_error)?;

    let mut dr = Decimal::ZERO;
    let mut cr = Decimal::ZERO;
    for balance in balances.values() {
      dr += balance.details.settled.dr_balance;
      cr += balance.details.settled.cr_balance;
    }

    Ok(JournalTotalsValues {
      journal_id: journal_id.to_string(),
      currency: currency.to_string(),
      dr: dr.to_string(),
      cr: cr.to_string(),
    })
  }
}
