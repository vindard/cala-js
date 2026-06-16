mod values;

use cala_ledger::primitives::{EntryId, TransactionId};

pub use values::EntryValues;

#[napi]
pub struct CalaEntry {
  inner: cala_ledger::entry::Entry,
}

#[napi]
impl CalaEntry {
  #[napi]
  pub fn id(&self) -> String {
    self.inner.id().to_string()
  }

  #[napi]
  pub fn values(&self) -> EntryValues {
    EntryValues::from(&self.inner)
  }
}

#[napi]
pub struct CalaEntries {
  inner: cala_ledger::entry::Entries,
}

#[napi]
impl CalaEntries {
  pub fn new(inner: &cala_ledger::entry::Entries) -> Self {
    Self {
      inner: inner.clone(),
    }
  }

  /// Returns entries for a transaction, sorted by `sequence` (so DEBIT
  /// rows come before their matched CREDIT, etc., as the template
  /// declared them).
  #[napi]
  pub async fn list_for_transaction_id(
    &self,
    transaction_id: String,
  ) -> napi::Result<Vec<CalaEntry>> {
    let tx_id = uuid::Uuid::parse_str(&transaction_id)
      .map(TransactionId::from)
      .map_err(crate::generic_napi_error)?;
    let entries = self
      .inner
      .list_for_transaction_id(tx_id)
      .await
      .map_err(crate::generic_napi_error)?;
    Ok(entries.into_iter().map(|e| CalaEntry { inner: e }).collect())
  }

  /// Bulk lookup by id. Order is not guaranteed by the underlying API;
  /// the returned vec follows the request order so callers can pair
  /// each id with its entry.
  #[napi]
  pub async fn find_all(&self, entry_ids: Vec<String>) -> napi::Result<Vec<CalaEntry>> {
    let ids: Vec<EntryId> = entry_ids
      .iter()
      .map(|s| {
        uuid::Uuid::parse_str(s)
          .map(EntryId::from)
          .map_err(crate::generic_napi_error)
      })
      .collect::<napi::Result<Vec<_>>>()?;

    let mut map = self
      .inner
      .find_all(&ids)
      .await
      .map_err(crate::generic_napi_error)?;

    // Reorder to match the caller's request order.
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
      if let Some(entry) = map.remove(&id) {
        out.push(CalaEntry { inner: entry });
      }
    }
    Ok(out)
  }
}
