use crate::error::RuntimeError;
use crate::models::{MonthAutopilotRequest, MonthAutopilotSummary};
use crate::runtime::ResolvedAutopilotBalances;
use crate::runtime::AppRuntime;
use logos_fetch::{FakeStatementAdapter, FetchRequest, FetchRunStatus, FetchedStatementArtifact, OnePasswordCliSecretResolver, OutputFormat, ProvidentAdapter, SecretBundle, SecretResolver, StatementAdapter, StatementSource, StatementSourceConfig};
use logos_store_aletheia::model::{StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus};
use std::env;
use std::fs;
use crate::runtime::LOGOS_FETCH_CONFIG_PATH_ENV;

impl AppRuntime {
    #[must_use]
    pub fn fetch_run(&self, run_id: &str) -> Option<StoredFetchRun> {
        self.store.fetch_run(run_id).cloned()
    }

    #[must_use]
    pub fn list_fetch_runs(
        &self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) -> Vec<StoredFetchRun> {
        let mut runs: Vec<_> = self
            .store
            .fetch_runs()
            .filter(|run| month_key.is_none_or(|month| run.month_key() == month))
            .filter(|run| checking_account.is_none_or(|account| run.ledger_account() == account))
            .cloned()
            .collect();
        runs.sort_by(|left, right| {
            right
                .created_at()
                .wallclock()
                .cmp(&left.created_at().wallclock())
                .then_with(|| left.run_id().cmp(right.run_id()))
        });
        runs
    }

    /// Stages fetched statement metadata for later month-autopilot resolution.
    pub fn stage_fetched_statement_artifact(&mut self, artifact: FetchedStatementArtifact) {
        self.fetched_statement_artifacts.retain(|existing| {
            !(existing.source_id() == artifact.source_id()
                && existing.ledger_account() == artifact.ledger_account()
                && existing.month_key() == artifact.month_key())
        });
        self.fetched_statement_artifacts.push(artifact);
    }

    /// Runs the full month workflow with close-time safety checks.
    ///
    /// Workflow:
    /// 1. optional statement PDF import
    /// 2. reconciliation preview + variance gate
    /// 3. reconciliation persistence
    /// 4. month report projection
    /// 5. immutable month close
    ///
    /// # Errors
    ///
    /// Returns an error when confirmation is missing, scope is already closed,
    /// variance safety checks fail, or any underlying import/store operation fails.
    pub fn run_month_autopilot(
        &mut self,
        request: &MonthAutopilotRequest,
    ) -> Result<MonthAutopilotSummary, RuntimeError> {
        if !request.confirm_close() {
            return Err(RuntimeError::Analytics {
                message: "month autopilot requires --confirm-close to persist month close"
                    .to_owned(),
            });
        }
        if self
            .month_close_for_scope(request.month_key(), request.checking_account())
            .is_some()
        {
            return Err(RuntimeError::Analytics {
                message: format!(
                    "month scope '{}' for '{}' is already closed",
                    request.month_key(),
                    request.checking_account()
                ),
            });
        }

        let needs_fetched_balances = !matches!(
            (
                request.opening_balance_cents(),
                request.closing_balance_cents(),
            ),
            (Some(_), Some(_))
        );
        let fetch_runs = if needs_fetched_balances
            && request.statement_pdf_path().is_none()
            && self
                .fetched_statement_artifact_for(request.checking_account(), request.month_key())
                .is_none()
        {
            self.fetch_configured_statement_artifacts(request, needs_fetched_balances)?
        } else {
            Vec::new()
        };

        let (imported_count, duplicate_count) = if let Some(path) = request.statement_pdf_path() {
            let summary = self.import_pdf_statement(
                path,
                request.checking_account(),
                false,
                request.enable_ocr(),
            )?;
            (summary.imported_count(), summary.duplicate_count())
        } else if let Some(artifact) =
            self.fetched_statement_artifact_for(request.checking_account(), request.month_key())
        {
            self.import_fetched_statement_artifact(&artifact, request.enable_ocr())?
        } else {
            (0, 0)
        };

        let balances = self.resolve_autopilot_balances(request)?;
        let preview = self.reconcile_month_for(
            request.checking_account(),
            request.month_key(),
            balances.opening_balance_cents,
            balances.closing_balance_cents,
        );
        if preview.variance_cents() != 0 && !request.allow_variance() {
            return Err(RuntimeError::Analytics {
                message: format!(
                    "month autopilot blocked close due to variance_cents={} (use --allow-variance to override)",
                    preview.variance_cents()
                ),
            });
        }
        let reconciled_txn_ids = self
            .reconciliation_transaction_ids_for(request.checking_account(), request.month_key());
        let matched_postings = i64::try_from(preview.matched_postings()).unwrap_or(i64::MAX);
        let (run, close) = self.store.write_reconciliation_run_and_month_close(
            request.month_key(),
            request.checking_account(),
            balances.opening_balance_cents,
            preview.ledger_delta_cents(),
            preview.expected_closing_balance_cents(),
            balances.closing_balance_cents,
            preview.variance_cents(),
            preview.is_reconciled(),
            matched_postings,
            preview.inflow_cents(),
            preview.outflow_cents(),
            &reconciled_txn_ids,
            request.analytics_artifact_id(),
        )?;
        let report = self.month_report_for(request.checking_account(), request.month_key());

        Ok(MonthAutopilotSummary::new(
            request.month_key(),
            request.checking_account(),
            imported_count,
            duplicate_count,
            fetch_runs,
            run,
            report,
            close,
        ))
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn fetch_configured_statement_artifacts(
        &mut self,
        request: &MonthAutopilotRequest,
        fetch_required: bool,
    ) -> Result<Vec<StoredFetchRun>, RuntimeError> {
        let config = match self.load_statement_source_config() {
            Ok(config) => config,
            Err(_err) if !fetch_required => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let Some(config) = config else {
            return Ok(Vec::new());
        };

        let matched_sources: Vec<_> = config
            .sources()
            .iter()
            .filter(|source| source.ledger_account() == request.checking_account())
            .cloned()
            .collect();
        if matched_sources.is_empty() {
            return Ok(Vec::new());
        }

        let fetch_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| RuntimeError::Analytics {
                message: format!("failed to start statement fetch runtime: {err}"),
            })?;

        let mut first_required_error = None;
        let mut staged_artifact = false;
        let mut persisted_runs = Vec::new();
        for source in matched_sources {
            let fetch_request = match FetchRequest::new(&source, request.month_key()) {
                Ok(fetch_request) => fetch_request,
                Err(err) => {
                    let err = fetch_error_to_runtime(&err);
                    if fetch_required && first_required_error.is_none() {
                        first_required_error = Some(RuntimeError::Analytics {
                            message: err.to_string(),
                        });
                    }
                    persisted_runs.push(self.persist_failed_fetch_run(
                        &source,
                        request.month_key(),
                        &err.to_string(),
                    )?);
                    continue;
                }
            };
            let secrets = match Self::secret_bundle_for_fetch_source(&source) {
                Ok(secrets) => secrets,
                Err(err) => {
                    if fetch_required && first_required_error.is_none() {
                        first_required_error = Some(RuntimeError::Analytics {
                            message: err.to_string(),
                        });
                    }
                    persisted_runs.push(self.persist_failed_fetch_run(
                        &source,
                        request.month_key(),
                        &err.to_string(),
                    )?);
                    continue;
                }
            };
            let result =
                match Self::run_fetch_adapter(&fetch_runtime, &source, &fetch_request, &secrets) {
                    Ok(result) => result,
                    Err(err) => {
                        if fetch_required && first_required_error.is_none() {
                            first_required_error = Some(RuntimeError::Analytics {
                                message: err.to_string(),
                            });
                        }
                        persisted_runs.push(self.persist_failed_fetch_run(
                            &source,
                            request.month_key(),
                            &err.to_string(),
                        )?);
                        continue;
                    }
                };
            if matches!(
                result.status(),
                FetchRunStatus::Downloaded | FetchRunStatus::Imported
            ) && result.artifact().is_none()
            {
                let message = format!(
                    "statement fetch for source '{}' returned {:?} without an artifact",
                    source.source_id(),
                    result.status()
                );
                if fetch_required && first_required_error.is_none() {
                    first_required_error = Some(RuntimeError::Analytics {
                        message: message.clone(),
                    });
                }
                persisted_runs.push(self.persist_failed_fetch_run(
                    &source,
                    request.month_key(),
                    &message,
                )?);
                continue;
            }
            match result.status() {
                FetchRunStatus::Downloaded | FetchRunStatus::Imported => {
                    if let Some(artifact) = result.artifact().cloned() {
                        self.stage_fetched_statement_artifact(artifact);
                        staged_artifact = true;
                    }
                    persisted_runs.push(self.persist_fetch_run_from_result(
                        &source,
                        request.month_key(),
                        &result,
                    )?);
                }
                FetchRunStatus::NoNewStatement => {
                    persisted_runs.push(self.persist_fetch_run_from_result(
                        &source,
                        request.month_key(),
                        &result,
                    )?);
                }
                FetchRunStatus::NeedsAttention | FetchRunStatus::Failed => {
                    persisted_runs.push(self.persist_fetch_run_from_result(
                        &source,
                        request.month_key(),
                        &result,
                    )?);
                    if fetch_required && first_required_error.is_none() {
                        first_required_error = Some(RuntimeError::Analytics {
                            message: format!(
                                "statement fetch for source '{}' requires attention before month autopilot can continue",
                                source.source_id()
                            ),
                        });
                    }
                }
            }
        }

        if fetch_required && !staged_artifact {
            if let Some(err) = first_required_error {
                return Err(err);
            }
        }

        Ok(persisted_runs)
    }

    pub(crate) fn fetched_statement_artifact_for(
        &self,
        checking_account: &str,
        month_key: &str,
    ) -> Option<FetchedStatementArtifact> {
        self.fetched_statement_artifacts
            .iter()
            .rev()
            .find(|artifact| {
                artifact.ledger_account() == checking_account && artifact.month_key() == month_key
            })
            .cloned()
    }

    pub(crate) fn import_fetched_statement_artifact(
        &mut self,
        artifact: &FetchedStatementArtifact,
        enable_ocr: bool,
    ) -> Result<(usize, usize), RuntimeError> {
        match artifact.output_format() {
            OutputFormat::Pdf => {
                let summary = self.import_pdf_statement(
                    artifact.artifact_path(),
                    artifact.ledger_account(),
                    false,
                    enable_ocr,
                )?;
                Ok((summary.imported_count(), summary.duplicate_count()))
            }
            OutputFormat::Csv => Err(RuntimeError::Analytics {
                message: format!(
                    "month autopilot cannot import fetched csv artifact '{}' yet",
                    artifact.artifact_path()
                ),
            }),
        }
    }

    pub(crate) fn resolve_autopilot_balances(
        &self,
        request: &MonthAutopilotRequest,
    ) -> Result<ResolvedAutopilotBalances, RuntimeError> {
        if let (Some(opening_balance_cents), Some(closing_balance_cents)) = (
            request.opening_balance_cents(),
            request.closing_balance_cents(),
        ) {
            return Ok(ResolvedAutopilotBalances {
                opening_balance_cents,
                closing_balance_cents,
            });
        }

        if let Some(artifact) =
            self.fetched_statement_artifact_for(request.checking_account(), request.month_key())
        {
            return Ok(ResolvedAutopilotBalances {
                opening_balance_cents: artifact.opening_balance_cents(),
                closing_balance_cents: artifact.closing_balance_cents(),
            });
        }

        Err(RuntimeError::Analytics {
            message:
                "month autopilot requires opening/closing balances or fetched statement metadata"
                    .to_owned(),
        })
    }

    pub(crate) fn load_statement_source_config(&self) -> Result<Option<StatementSourceConfig>, RuntimeError> {
        let Some(path) = self.fetch_config_path.as_ref() else {
            return Ok(None);
        };
        if !path.exists() {
            if env::var_os(LOGOS_FETCH_CONFIG_PATH_ENV).is_some() {
                return Err(RuntimeError::Analytics {
                    message: format!(
                        "statement fetch config path '{}' does not exist",
                        path.display()
                    ),
                });
            }
            return Ok(None);
        }

        let input = fs::read_to_string(path).map_err(|err| RuntimeError::Analytics {
            message: format!(
                "failed to read statement source config '{}': {err}",
                path.display()
            ),
        })?;

        StatementSourceConfig::from_toml(&input)
            .map(Some)
            .map_err(|err| fetch_error_to_runtime(&err))
    }

    pub(crate) fn secret_bundle_for_fetch_source(
        source: &StatementSource,
    ) -> Result<SecretBundle, RuntimeError> {
        let resolver = OnePasswordCliSecretResolver::from_environment();
        Self::secret_bundle_for_fetch_source_with_resolver(source, &resolver)
    }

    pub(crate) fn secret_bundle_for_fetch_source_with_resolver<R>(
        source: &StatementSource,
        resolver: &R,
    ) -> Result<SecretBundle, RuntimeError>
    where
        R: SecretResolver,
    {
        match source.institution_id() {
            "fake-fixture" => SecretBundle::new("fixture-user", "fixture-pass", Some("000000"))
                .map_err(|err| fetch_error_to_runtime(&err)),
            "fake-needs-attention" => {
                SecretBundle::new("fixture-user", "fixture-pass", Some("000000"))
                    .map_err(|err| fetch_error_to_runtime(&err))
            }
            _ => resolver
                .resolve(source)
                .map_err(|err| fetch_error_to_runtime(&err)),
        }
    }

    pub(crate) fn run_fetch_adapter(
        fetch_runtime: &tokio::runtime::Runtime,
        source: &StatementSource,
        request: &FetchRequest,
        secrets: &SecretBundle,
    ) -> Result<logos_fetch::FetchResult, RuntimeError> {
        match source.institution_id() {
            "fake-fixture" => fetch_runtime
                .block_on(
                    FakeStatementAdapter::download_fixture_statement().fetch(request, secrets),
                )
                .map_err(|err| fetch_error_to_runtime(&err)),
            "fake-needs-attention" => fetch_runtime
                .block_on(
                    FakeStatementAdapter::needs_attention("mfa challenge required")
                        .fetch(request, secrets),
                )
                .map_err(|err| fetch_error_to_runtime(&err)),
            "provident-credit-union" => fetch_runtime
                .block_on(ProvidentAdapter::fixture_runner_output().fetch(request, secrets))
                .map_err(|err| fetch_error_to_runtime(&err)),
            institution_id => Err(RuntimeError::Analytics {
                message: format!(
                    "no statement fetch adapter is registered for institution \'{institution_id}\'"
                ),
            }),
        }
    }

    pub(crate) fn persist_fetch_run_from_result(
        &mut self,
        source: &StatementSource,
        month_key: &str,
        result: &logos_fetch::FetchResult,
    ) -> Result<StoredFetchRun, RuntimeError> {
        let artifact = result.artifact();
        self.store
            .write_fetch_run(
                source.source_id(),
                source.institution_id(),
                source.ledger_account(),
                month_key,
                store_fetch_run_status(result.status()),
                artifact.map(logos_fetch::FetchedStatementArtifact::artifact_path),
                artifact.map(|value| output_format_label(value.output_format())),
                artifact.map(logos_fetch::FetchedStatementArtifact::opening_balance_cents),
                artifact.map(logos_fetch::FetchedStatementArtifact::closing_balance_cents),
                result.error_summary(),
            )
            .map_err(RuntimeError::from)
    }

    pub(crate) fn persist_failed_fetch_run(
        &mut self,
        source: &StatementSource,
        month_key: &str,
        error_summary: &str,
    ) -> Result<StoredFetchRun, RuntimeError> {
        self.store
            .write_fetch_run(
                source.source_id(),
                source.institution_id(),
                source.ledger_account(),
                month_key,
                StoredFetchRunStatus::Failed,
                None,
                None,
                None,
                None,
                Some(error_summary),
            )
            .map_err(RuntimeError::from)
    }

}

pub(crate) fn fetch_error_to_runtime(err: &logos_fetch::FetchError) -> RuntimeError {
    RuntimeError::Analytics {
        message: err.to_string(),
    }
}

pub(crate) const fn output_format_label(output_format: OutputFormat) -> StoredFetchArtifactFormat {
    match output_format {
        OutputFormat::Csv => StoredFetchArtifactFormat::Csv,
        OutputFormat::Pdf => StoredFetchArtifactFormat::Pdf,
    }
}

pub(crate) const fn store_fetch_run_status(status: FetchRunStatus) -> StoredFetchRunStatus {
    match status {
        FetchRunStatus::Downloaded => StoredFetchRunStatus::Downloaded,
        FetchRunStatus::Imported => StoredFetchRunStatus::Imported,
        FetchRunStatus::NoNewStatement => StoredFetchRunStatus::NoNewStatement,
        FetchRunStatus::NeedsAttention => StoredFetchRunStatus::NeedsAttention,
        FetchRunStatus::Failed => StoredFetchRunStatus::Failed,
    }
}
