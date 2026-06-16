mod config;

pub use config::*;

use napi::bindgen_prelude::AsyncTask;
use napi::{Env, Task};

use crate::{
  balance::CalaBalances, entry::CalaEntries, transaction::CalaTransactions,
  tx_template::CalaTxTemplates,
};

use super::{account::*, journal::*};

#[napi]
pub struct CalaLedger {
  inner: cala_ledger::CalaLedger,
}

pub struct ConnectTask {
  config: CalaLedgerConfig,
}

impl Task for ConnectTask {
  type Output = cala_ledger::CalaLedger;
  type JsValue = CalaLedger;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    use cala_ledger::CalaLedgerConfig as Config;
    let mut builder = Config::builder();
    builder
      .pg_con(self.config.pg_con.clone())
      .exec_migrations(true);
    if let Some(n) = self.config.max_connections {
      builder.max_connections(n);
    }
    let config = builder.build().map_err(crate::generic_napi_error)?;
    napi::bindgen_prelude::block_on(cala_ledger::CalaLedger::init(config))
      .map_err(crate::generic_napi_error)
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    Ok(CalaLedger { inner: output })
  }
}

#[napi]
impl CalaLedger {
  #[napi(ts_return_type = "Promise<CalaLedger>")]
  pub fn connect(config: CalaLedgerConfig) -> AsyncTask<ConnectTask> {
    AsyncTask::new(ConnectTask { config })
  }

  #[napi]
  pub fn accounts(&self) -> napi::Result<CalaAccounts> {
    Ok(CalaAccounts::new(self.inner.accounts()))
  }

  #[napi]
  pub fn journals(&self) -> napi::Result<CalaJournals> {
    Ok(CalaJournals::new(self.inner.journals()))
  }

  #[napi]
  pub fn tx_templates(&self) -> napi::Result<CalaTxTemplates> {
    Ok(CalaTxTemplates::new(self.inner.tx_templates()))
  }

  #[napi]
  pub fn transactions(&self) -> napi::Result<CalaTransactions> {
    Ok(CalaTransactions::new(
      self.inner.transactions(),
      &self.inner,
    ))
  }

  #[napi]
  pub fn entries(&self) -> napi::Result<CalaEntries> {
    Ok(CalaEntries::new(self.inner.entries()))
  }

  #[napi]
  pub fn balances(&self) -> napi::Result<CalaBalances> {
    Ok(CalaBalances::new(self.inner.balances()))
  }
}
