use cala_ledger::primitives::{DebitOrCredit, Layer};

#[napi(object)]
pub struct EntryValues {
  pub id: String,
  pub transaction_id: String,
  pub journal_id: String,
  pub account_id: String,
  pub entry_type: String,
  pub sequence: u32,
  pub layer: String,
  pub units: String,
  pub currency: String,
  pub direction: String,
  pub description: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

fn layer_str(l: Layer) -> &'static str {
  match l {
    Layer::Settled => "SETTLED",
    Layer::Pending => "PENDING",
    Layer::Encumbrance => "ENCUMBRANCE",
  }
}

fn direction_str(d: DebitOrCredit) -> &'static str {
  match d {
    DebitOrCredit::Debit => "DEBIT",
    DebitOrCredit::Credit => "CREDIT",
  }
}

impl From<&cala_ledger::entry::Entry> for EntryValues {
  fn from(entry: &cala_ledger::entry::Entry) -> Self {
    let v = entry.values().clone();
    Self {
      id: v.id.to_string(),
      transaction_id: v.transaction_id.to_string(),
      journal_id: v.journal_id.to_string(),
      account_id: v.account_id.to_string(),
      entry_type: v.entry_type,
      sequence: v.sequence,
      layer: layer_str(v.layer).to_string(),
      units: v.units.to_string(),
      currency: v.currency.to_string(),
      direction: direction_str(v.direction).to_string(),
      description: v.description,
      metadata: v.metadata,
    }
  }
}

impl From<cala_ledger::entry::Entry> for EntryValues {
  fn from(entry: cala_ledger::entry::Entry) -> Self {
    Self::from(&entry)
  }
}
