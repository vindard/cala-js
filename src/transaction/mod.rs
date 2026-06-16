mod values;

use cala_ledger::velocity::Params;
use cala_ledger::TransactionId;
use cala_types::param::ParamDataType;
use cel_interpreter::CelValue;
use chrono::DateTime;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use values::*;

use crate::query::*;

#[napi]
pub struct CalaTransaction {
  inner: cala_ledger::transaction::Transaction,
}

#[napi]
impl CalaTransaction {
  #[napi]
  pub fn id(&self) -> String {
    self.inner.id().to_string()
  }

  #[napi]
  pub fn values(&self) -> TransactionValues {
    TransactionValues::from(&self.inner)
  }
}

#[napi(object)]
pub struct PaginatedTransactions {
  pub transactions: Vec<TransactionValues>,
  pub has_next_page: bool,
  pub end_cursor: Option<CursorToken>,
}

#[napi]
pub struct CalaTransactions {
  inner: cala_ledger::transaction::Transactions,
  ledger: cala_ledger::CalaLedger,
}

#[napi]
impl CalaTransactions {
  pub fn new(
    inner: &cala_ledger::transaction::Transactions,
    inner_ledger: &cala_ledger::CalaLedger,
  ) -> Self {
    Self {
      inner: inner.clone(),
      ledger: inner_ledger.clone(),
    }
  }

  #[napi]
  pub async fn find_by_id(&self, transaction_id: String) -> napi::Result<CalaTransaction> {
    let transaction_id =
      uuid::Uuid::parse_str(&transaction_id).map_err(crate::generic_napi_error)?;

    let transaction_id = TransactionId::from(transaction_id);

    let transaction = self
      .inner
      .find_by_id(transaction_id)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaTransaction { inner: transaction })
  }

  #[napi]
  pub async fn find_by_external_id(&self, external_id: String) -> napi::Result<CalaTransaction> {
    let transaction = self
      .inner
      .find_by_external_id(external_id)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaTransaction { inner: transaction })
  }

  #[napi]
  pub async fn void_transaction(&self, existing_tx_id: String) -> napi::Result<CalaTransaction> {
    let voiding_tx_id = TransactionId::new();

    let existing_tx_id =
      uuid::Uuid::parse_str(&existing_tx_id).map_err(crate::generic_napi_error)?;

    let existing_tx_id = TransactionId::from(existing_tx_id);

    let transaction = self
      .ledger
      .void_transaction(voiding_tx_id, existing_tx_id)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaTransaction { inner: transaction })
  }

  /// List transactions posted through a tx-template, paginated. Returns
  /// most-recent-first (descending by created_at). The frontend reverses
  /// for display order. Cursor token is the standard PaginatedQueryArgs
  /// cursor shape — opaque to JS.
  #[napi]
  pub async fn list_for_template_code(
    &self,
    tx_template_code: String,
    query: PaginatedQueryArgs,
  ) -> napi::Result<PaginatedTransactions> {
    let tx_template = self
      .ledger
      .tx_templates()
      .find_by_code(tx_template_code)
      .await
      .map_err(crate::generic_napi_error)?;
    let template_id = tx_template.id();

    let query = cala_ledger::es_entity::PaginatedQueryArgs {
      after: query.after.map(|c| c.try_into()).transpose()?,
      first: usize::try_from(query.first).map_err(crate::generic_napi_error)?,
    };
    let ret = self
      .inner
      .list_for_template_id(
        template_id,
        query,
        cala_ledger::es_entity::ListDirection::Descending,
      )
      .await
      .map_err(crate::generic_napi_error)?;
    Ok(PaginatedTransactions {
      transactions: ret
        .entities
        .iter()
        .map(TransactionValues::from)
        .collect(),
      has_next_page: ret.has_next_page,
      end_cursor: ret.end_cursor.map(|c| c.into()),
    })
  }

  #[napi]
  pub async fn post(
    &self,
    tx_template_code: String,
    params: JsonValue,
  ) -> napi::Result<CalaTransaction> {
    let transaction_id = TransactionId::new();

    let tx_template = self
      .ledger
      .tx_templates()
      .find_by_code(tx_template_code.clone())
      .await
      .map_err(crate::generic_napi_error)?;

    let template_params = tx_template.values().params.clone();

    let mut param_types_map: HashMap<String, ParamDataType> = HashMap::new();

    if let Some(params) = &template_params {
      for param in params {
        param_types_map.insert(param.name.clone(), param.r#type.clone());
      }
    }

    let params_object: HashMap<String, JsonValue> = serde_json::from_value(params)
      .map_err(|e| napi::Error::from_reason(format!("Failed to parse parameters: {}", e)))?;

    let mut cala_params: Params = Params::new();

    for (key, value) in params_object {
      let hint = param_types_map.get(&key).cloned();
      let cel_value = value.to_cel_value(hint).map_err(|e| {
        napi::Error::from_reason(format!("Failed to convert parameter '{}': {}", key, e))
      })?;

      cala_params.insert(key, cel_value);
    }

    let transaction = self
      .ledger
      .post_transaction(transaction_id, &tx_template_code, cala_params)
      .await
      .map_err(crate::generic_napi_error)?;

    Ok(CalaTransaction { inner: transaction })
  }
}

trait ToCelValue {
  fn to_cel_value(self, hint: Option<ParamDataType>) -> napi::Result<CelValue>;
}

impl ToCelValue for JsonValue {
  fn to_cel_value(self, hint: Option<ParamDataType>) -> napi::Result<CelValue> {
    match (hint, self) {
      (Some(ParamDataType::Decimal), JsonValue::Number(n)) => {
        let decimal = Decimal::from_f64(n.as_f64().ok_or_else(|| {
          napi::Error::from_reason("Failed to convert number to f64 for Decimal".to_string())
        })?)
        .ok_or_else(|| {
          napi::Error::from_reason(format!("Failed to convert float {} to Decimal", n))
        })?;

        Ok(CelValue::from(decimal))
      }
      (Some(ParamDataType::Date), JsonValue::String(s)) => {
        let parsed_date = DateTime::parse_from_rfc3339(&s)
          .map(|dt| dt.naive_utc().date())
          .map_err(|e| napi::Error::from_reason(format!("Invalid date format: {}", e)))?;

        Ok(CelValue::from(parsed_date))
      }
      (_, value) => Ok(CelValue::from(value)),
    }
  }
}
