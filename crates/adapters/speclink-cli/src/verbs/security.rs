//! Device-local data management. No workspace/remote handshake is required.
use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use speclink_core::security::{
    data::StandardRepository,
    prepared::{PreparedRepository, ReviewBinding},
    standards::StandardId,
};
use speclink_host::security::{self, ReviewBudget, ReviewTarget};
use speclink_host::security_data::{self, DataJob, OfficialTransport};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Args)]
pub(crate) struct SecurityArgs {
    #[command(subcommand)]
    command: SecurityCommand,
}
#[derive(Subcommand)]
enum SecurityCommand {
    /// Submit or inspect deep security reviews attached to a proposal
    Review {
        #[command(subcommand)]
        command: ReviewCommand,
    },
    /// Prepare the local knowledge snapshot from already installed data
    Kb {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
    /// Run built-in preliminary rules entirely offline
    Run(TargetArgs),
    /// Explicitly prepare policy and SDD artifacts for offline review
    Prepare(TargetArgs),
    /// List the SpecLink-owned rules
    Rules {
        #[arg(long)]
        json: bool,
    },
    /// Read an existing redacted report without rescanning
    Report {
        id: String,
        #[arg(long)]
        change: Option<String>,
        #[arg(long, value_enum, default_value = "json")]
        format: Format,
    },
    /// Manage official standard data stored on this device
    Data {
        #[command(subcommand)]
        command: DataCommand,
    },
}
#[derive(Subcommand)]
enum ReviewCommand {
    Accept {
        change: String,
        #[arg(long, required=true)]
        stdin: bool,
        #[arg(long)]
        json: bool,
    },
    AddRound {
        change: String,
        #[arg(long, required=true)]
        stdin: bool,
        #[arg(long)]
        json: bool,
    },
    Show { change: String, #[arg(long)] json: bool },
}
#[derive(serde::Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct LegacyReportImport {
    legacy_import_report_id: String,
}
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ReviewRoundInput {
    Legacy(LegacyReportImport),
    Deep(speclink_core::security::report::DeepReportInput),
}
#[derive(Subcommand)]
enum KnowledgeCommand {
    /// Build .security_kb without downloading resources
    Prepare {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        change: Option<String>,
        #[arg(long)]
        json: bool,
    },
}
#[derive(Args)]
struct TargetArgs {
    #[arg(long)]
    root: Option<PathBuf>,
    #[arg(long)]
    change: Option<String>,
    #[arg(long)]
    json: bool,
}
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Markdown,
}

#[derive(Debug)]
struct SecurityExit(u8);
impl std::fmt::Display for SecurityExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "security execution ended with status {}", self.0)
    }
}
impl std::error::Error for SecurityExit {}
pub(crate) fn exit_status(error: &anyhow::Error) -> Option<u8> {
    error.downcast_ref::<SecurityExit>().map(|e| e.0)
}
#[derive(Subcommand)]
enum DataCommand {
    /// List supported and installed versions without connecting to the network
    List {
        #[arg(long)]
        json: bool,
    },
    /// Explicitly check official structured metadata for supported versions
    Refresh {
        #[arg(long)]
        json: bool,
    },
    /// Download and validate a supported official version
    Install {
        standard: StandardId,
        #[arg(long)]
        version: String,
        #[arg(long)]
        reinstall: bool,
        #[arg(long)]
        json: bool,
    },
    /// Install an explicitly selected offline package
    Import {
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Export installed source data for offline use (no network access)
    Export {
        #[arg(
            required_unless_present = "all",
            conflicts_with = "all",
            requires = "version"
        )]
        standard: Option<StandardId>,
        #[arg(long, requires = "standard", conflicts_with = "all")]
        version: Option<String>,
        #[arg(long, conflicts_with = "standard")]
        all: bool,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Remove a version unused by locally known policies and active readers
    Remove {
        standard: StandardId,
        #[arg(long)]
        version: String,
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn cmd_security(args: SecurityArgs) -> Result<()> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Relaxed))
        .map_err(|_| anyhow::anyhow!("cancel_handler_unavailable"))?;
    let result = security_command(args, cancelled.clone());
    if result.is_err() && cancelled.load(Ordering::Relaxed) {
        return Err(SecurityExit(130).into());
    }
    result
}
fn security_command(args: SecurityArgs, cancelled: Arc<AtomicBool>) -> Result<()> {
    if let SecurityCommand::Rules { json } = &args.command {
        let rules = speclink_core::security::rules::RuleSet::builtin()?;
        if *json {
            return crate::common::print_json(&rules);
        }
        println!("Rule version: {}", rules.rule_version);
        for rule in rules.rules {
            println!(
                "{} · {} · {} · {:?}\n{}\n{}",
                rule.id,
                rule.title,
                rule.severity,
                rule.languages,
                rule.description,
                rule.remediation
            );
        }
        return Ok(());
    }
    let application_data = speclink_host::context::security_data_dir()?;
    if let SecurityCommand::Report { id, change, format } = &args.command {
        if let Some(change) = change {
            let location = target_location(&TargetArgs {root:None,change:Some(change.clone()),json:false})?;
            if location.remote.is_some() {
                let report=remote_call(&location,|client|client.security_change_report(change,id))?;
                let report=speclink_core::security::report::VersionedReport::from_json(&serde_json::to_vec(&report)?)?;
                println!("{}",match format {Format::Json=>report.json()?,Format::Markdown=>report.markdown()});return Ok(());
            }
            let store = speclink_fs::FsStore::new(location.reader.root(), &location.spec_dir);
            let report = speclink_core::security::report::read_change_report(&store,change,id)?.ok_or_else(||anyhow::anyhow!("report_not_found"))?;
            println!("{}", match format { Format::Json=>report.json()?,Format::Markdown=>report.markdown() });
            return Ok(());
        }
        let reports =
            speclink_fs::security::reports::FsSecurityReports::new(&application_data)?;
        let text = security::export_report(
            &reports,
            id,
            match format {
                Format::Json => security::ReportFormat::Json,
                Format::Markdown => security::ReportFormat::Markdown,
            },
        )?;
        println!("{text}");
        return Ok(());
    }
    let job = DataJob::new(cancelled.clone());
    let progress = |p: speclink_core::security::JobProgress| {
        eprintln!(
            "{}: {} bytes, {} files",
            p.phase, p.completed_bytes, p.completed_files
        );
    };
    match args.command {
        SecurityCommand::Review { command } => {
            let repository=crate::common::security_repository(&application_data).ok();
            review_command(command, &application_data, repository.as_ref().map(|repo| repo as &dyn StandardRepository), cancelled)
        },
        SecurityCommand::Kb { command: KnowledgeCommand::Prepare { root, change, json } } => {
            let repository = crate::common::security_repository(&application_data)?;
            let args = TargetArgs { root, change: change.clone(), json };
            let location = target_location(&args)?;
            let prepared = speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
            let target = location.review_target(&prepared, change.clone())?;
            let cache_key=target.binding.project_key();
            let key = target.knowledge_project_key();
            let policy_text = if target.binding.is_remote() {
                prepared.load_policy(&cache_key, change.as_deref())?.ok_or_else(|| anyhow::anyhow!("missing_preparation"))?.policy_text
            } else {
                location.reader.read_optional_text(&format!("{}/config.yaml", location.spec_dir), 64 * 1024, &|| None)?.unwrap_or_default()
            };
            let receipt = speclink_fs::security::knowledge::prepare(location.reader.root(), &key, &policy_text, &repository, &|| cancelled.load(Ordering::Relaxed))?;
            if json { crate::common::print_json(&receipt) } else {
                println!("Prepared knowledge at {} · {} files, {} bytes · {}", receipt.path, receipt.files, receipt.bytes, receipt.knowledge_digest);
                println!("projectKey: {}\npolicyDigest: {}", receipt.project_key, receipt.policy_digest);
                for standard in &receipt.standards {
                    println!("standardId: {} · version: {} · dataSha256: {}", standard.standard_id.as_str(), standard.version, standard.data_sha256);
                }
                Ok(())
            }
        }
        SecurityCommand::Rules { .. } => unreachable!(),
        SecurityCommand::Report { .. } => unreachable!(),
        SecurityCommand::Run(args) => {
            let repository = crate::common::security_repository(&application_data)?;
            let prepared =
                speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
            let location = target_location(&args)?;
            let target = location.review_target(&prepared, args.change.clone())?;
            let reports =
                speclink_fs::security::reports::FsSecurityReports::new(&application_data)?;
            let store = speclink_fs::FsStore::new(location.reader.root(), &location.spec_dir);
            let proposal = args.change.as_deref().filter(|_|location.remote.is_none()).map(|change|security::ProposalReports {store:&store,change});
            let selected_reports: &dyn speclink_core::security::report::ReportRepository = proposal.as_ref().map(|p|p as _).unwrap_or(&reports);
            let outcome = security::run_review(
                &target,
                Some(&location.reader),
                &repository,
                &prepared,
                selected_reports,
                &ReviewBudget::new(cancelled),
                &progress,
            )?;
            if args.json {
                crate::common::print_json(&outcome.report)?;
            } else {
                println!("{}", outcome.report.summary());
            }
            if location.remote.is_some()&&args.change.is_some(){eprintln!("pending_publication: saved a device draft; explicitly publish it with security review add-round");}
            for warning in outcome.receipt.warnings {
                eprintln!("{warning}");
            }
            if outcome.report.is_cancelled() {
                Err(SecurityExit(130).into())
            } else if !outcome.report.is_complete() {
                Err(SecurityExit(1).into())
            } else {
                Ok(())
            }
        }
        SecurityCommand::Prepare(args) => {
            let prepared =
                speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
            let location = target_location(&args)?;
            let budget = ReviewBudget::new(cancelled);
            let (target, input) = if let Some(remote) = &location.remote {
                remote_preparation(&location, remote, args.change.clone(), &budget)?
            } else {
                let target = ReviewTarget::local(
                    location.reader.root(),
                    &location.spec_dir,
                    args.change.clone(),
                )?;
                let input = security::local_preparation(&target, &location.reader, &budget)?;
                (target, input)
            };
            let receipt =
                security::prepare_review(&target, &|| Ok(input.clone()), &prepared, &budget)?;
            if args.json {
                crate::common::print_json(&receipt)
            } else {
                println!(
                    "Prepared {} at {} · {} documents",
                    receipt.project_key, receipt.prepared_at, receipt.document_count
                );
                Ok(())
            }
        }
        SecurityCommand::Data { command } => {
          let repository = crate::common::security_repository(&application_data)?;
          match command {
            DataCommand::List { json } => {
                print_inventory(security_data::list_data(&repository)?, json)
            }
            DataCommand::Refresh { json } => {
                let result = security_data::refresh_data(
                    &repository,
                    &OfficialTransport::new(&job),
                    &job,
                    &progress,
                )?;
                print_inventory(result, json)
            }
            DataCommand::Install {
                standard,
                version,
                reinstall,
                json,
            } => {
                let receipt = security_data::install_data(
                    &repository,
                    standard,
                    &version,
                    reinstall,
                    &OfficialTransport::new(&job),
                    &speclink_fs::security::decode_sources,
                    &job,
                    &progress,
                )?;
                if json {
                    crate::common::print_json(&receipt)
                } else {
                    println!(
                        "{} {}: {}\nsha256: {}\nsource revision: {}\nintegrity: {:?}",
                        standard.as_str(),
                        version,
                        receipt.installed_status,
                        receipt.sha256,
                        receipt.source_revision,
                        receipt.integrity_kind
                    );
                    for warning in receipt.warnings {
                        eprintln!("{warning}");
                    }
                    Ok(())
                }
            }
            DataCommand::Export {
                standard,
                version,
                all,
                output,
                json,
            } => {
                let selection = if all {
                    security_data::ExportSelection::All
                } else {
                    security_data::ExportSelection::One(
                        standard.expect("clap requires standard"),
                        version.expect("clap requires version"),
                    )
                };
                let receipt = security_data::export_data(
                    &repository,
                    selection,
                    &job,
                    &progress,
                    &|selected, all, skipped, load| {
                        speclink_fs::security_packages::write_export(
                            &output,
                            selected,
                            all,
                            skipped,
                            load,
                            &|| job.check().is_err(),
                        )
                    },
                )?;
                if json {
                    crate::common::print_json(&receipt)
                } else {
                    println!(
                        "Exported {} packages to {} ({} bytes)",
                        receipt.packages.len(),
                        receipt.path,
                        receipt.bytes
                    );
                    for package in receipt.packages {
                        println!(
                            "{} {}: {}",
                            package.standard_id.as_str(),
                            package.version,
                            package.sha256
                        );
                    }
                    for skipped in receipt.skipped {
                        println!(
                            "Skipped {} {}: {}",
                            skipped.standard_id.as_str(),
                            skipped.version,
                            skipped.reason
                        );
                    }
                    Ok(())
                }
            }
            DataCommand::Import { path, json } => {
                use speclink_core::security::data::ImportReceipt;
                let receipt = security_data::import_data_bundle(
                    &repository,
                    &path,
                    &job,
                    &progress,
                    &speclink_fs::security_packages::prepare_collection,
                )?;
                match receipt {
                    ImportReceipt::Single(receipt) => {
                        if json {
                            crate::common::print_json(&receipt)
                        } else {
                            println!(
                                "{} {}: {}\nsha256: {}\nsource revision: {}\nintegrity: {:?}",
                                receipt.standard_id.as_str(),
                                receipt.version,
                                receipt.installed_status,
                                receipt.sha256,
                                receipt.source_revision,
                                receipt.integrity_kind
                            );
                            Ok(())
                        }
                    }
                    ImportReceipt::Batch(receipt) => {
                        if json {
                            crate::common::print_json(&receipt)?;
                        } else {
                            println!("Collection import: {}", receipt.status);
                            for result in &receipt.results {
                                println!(
                                    "{} {}: {}{}",
                                    result.standard_id.as_str(),
                                    result.version,
                                    result.status,
                                    result
                                        .error_code
                                        .as_ref()
                                        .map(|code| format!(" ({code})"))
                                        .unwrap_or_default()
                                );
                                if let Some(sha256) = &result.sha256 {
                                    println!("sha256: {sha256}");
                                }
                            }
                        }
                        let status = batch_exit_code(&receipt.status);
                        if status == 0 {
                            Ok(())
                        } else {
                            eprintln!(
                                "{}: collection import stopped",
                                if status == 130 {
                                    "cancelled"
                                } else {
                                    "import_failed"
                                }
                            );
                            Err(SecurityExit(status).into())
                        }
                    }
                }
            }
            DataCommand::Remove {
                standard,
                version,
                json,
            } => {
                remember_current_policy(&repository)?;
                let result = security_data::remove_data(&repository, standard, &version)?;
                if json {
                    crate::common::print_json(&result)
                } else {
                    println!(
                        "{} {}: removed\n{}",
                        standard.as_str(),
                        version,
                        result.reference_scope
                    );
                    Ok(())
                }
            }
          }
        },
    }
}

fn print_inventory(data: security_data::StandardDataList, json: bool) -> Result<()> {
    if json {
        return crate::common::print_json(&data);
    }
    for standard in data.standards {
        for version in standard.versions {
            println!(
                "{} {}: {}\n  {}\n  {}: {}\n  {}\n  format: {}",
                standard.standard_id.as_str(),
                version.version,
                version.status,
                version.source_url,
                version.license_name,
                version.license_url,
                version.attribution,
                version.source_format
            );
            if let Some(error) = version.error {
                eprintln!("{error}");
            }
        }
    }
    Ok(())
}

fn review_command(command: ReviewCommand, application_data: &std::path::Path, data: Option<&dyn StandardRepository>, cancelled: Arc<AtomicBool>) -> Result<()> {
    use speclink_core::{security::report::*, store::Store};
    let (change,json) = match &command {ReviewCommand::Show{change,json}|ReviewCommand::AddRound{change,json,..}|ReviewCommand::Accept{change,json,..}=>(change.clone(),*json)};
    let location = target_location(&TargetArgs {root:None,change:Some(change.clone()),json})?;
    if location.remote.is_some(){return remote_review_command(command,&location,application_data,data,cancelled);}
    speclink_core::security::prepared::validate_change(Some(&change))?;
    let store = speclink_fs::FsStore::new(location.reader.root(), &location.spec_dir);
    speclink_core::history::index(&store,&change)?;
    let prepared = speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?;
    let target = location.review_target(&prepared,Some(change.clone()))?;
    match command {
        ReviewCommand::Accept {stdin:_,..} => {
            let data=data.ok_or_else(||anyhow::anyhow!("standard_data_unavailable"))?;
            use std::io::Read;
            let mut bytes=Vec::new();std::io::stdin().take(16*1024+1).read_to_end(&mut bytes)?;
            anyhow::ensure!(bytes.len()<=16*1024,"size_limit: security acceptance input");
            let input:AcceptRiskInput=serde_json::from_slice(&bytes).map_err(|_|anyhow::anyhow!("invalid_security_decision: input JSON"))?;
            let receipt=security::accept_current_risk(&target,&location.reader,data,&store,input,speclink_host::context::git_identity(location.reader.root()),&ReviewBudget::new(cancelled))?;
            if json {crate::common::print_json(&receipt)} else {println!("Accepted security risk {} · report {} · finding {}",receipt.decision_id,receipt.report_id,receipt.finding_id);Ok(())}
        },
        ReviewCommand::AddRound {stdin:_,..} => {
            let data=data.ok_or_else(||anyhow::anyhow!("standard_data_unavailable"))?;
            use std::io::Read;
            let mut bytes=Vec::new(); std::io::stdin().take(4*1024*1024+1).read_to_end(&mut bytes)?;
            anyhow::ensure!(bytes.len()<=4*1024*1024,"size_limit: security report input");
            let input:ReviewRoundInput=serde_json::from_slice(&bytes).map_err(|_|anyhow::anyhow!("invalid_report: submission JSON"))?;
            let input=match input {
              ReviewRoundInput::Legacy(input) => {
                anyhow::ensure!(store.change_exists(&change), "unknown_change");
                let reports = speclink_fs::security::reports::FsSecurityReports::new(application_data)?;
                let report=reports.get(&input.legacy_import_report_id)?.ok_or_else(||anyhow::anyhow!("report_not_found"))?;
                anyhow::ensure!(report.project_key()==target.binding.project_key() && report.change_name()==Some(change.as_str()),"binding_mismatch");
                save_legacy_change_report(&store,&change,&report)?;
                let result=serde_json::json!({"reportId":report.report_id(),"changeName":change,"relativePath":format!("checks/security/{}/report.md",report.report_id()),"complete":report.is_complete()});
                if json { return crate::common::print_json(&result); }
                println!("Imported legacy report {} · excluded from acceptance",report.report_id()); return Ok(());
              }
              ReviewRoundInput::Deep(input) => input,
            };
            let (context,builtin)=security::submission_context(&target,&location.reader,data,&store,speclink_host::context::git_identity(location.reader.root()),&ReviewBudget::new(cancelled))?;
            let receipt=security::add_deep_round(&store,&change,input,context,builtin)?;
            if json {crate::common::print_json(&receipt)} else {println!("Saved security review {} · {} · complete: {}",receipt.report_id,receipt.relative_path,receipt.complete);Ok(())}
        },
        ReviewCommand::Show {..} => {
            let policy_text=location.reader.read_optional_text(&format!("{}/config.yaml",location.spec_dir),64*1024,&||None)?;
            let policy = speclink_core::config::WorkflowConfig::from_text(policy_text.as_deref()).map_err(|_|anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?.security();
            let mut reason=None;
            let mut fresh=None;
            let submission=if store.change_exists(&change) {
                if let Some(data)=data {
                  match security::submission_context(&target,&location.reader,data,&store,None,&ReviewBudget::new(cancelled)) {
                    Ok((context,builtin))=>{fresh=Some(security::fresh_evidence(&context));let mut s=security::Submission::from(&context);s.builtin_report_id=builtin;Some(s)},
                    Err(error)=>{reason=Some(error.to_string().split(':').next().unwrap_or("source_freshness_unverified").to_owned());None}
                  }
                } else {reason=Some("standard_data_unavailable".into());None}
            } else {None};
            let mut result=security::review_view(&store,&change,&target.binding.project_key(),&policy,fresh.as_ref())?;
            result.submission=submission.map(|s|serde_json::from_value(serde_json::to_value(s)?)).transpose()?;
            if let Some(reason)=reason {if !result.acceptance.reasons.contains(&reason){result.acceptance.reasons.push(reason);}}
            print_security_review(&result,json)
        }
    }
}

fn print_security_review(view: &speclink_protocol::security::SecurityReviewView, json: bool) -> Result<()> {
    if json { return crate::common::print_json(view); }
    println!("Security reviews for {}", view.change_name);
    for report in &view.reports {
        println!("{} · {} · {} · complete: {} · legacy: {}", report.report_id, report.execution_mode, report.relative_path, report.complete, report.legacy);
    }
    let acceptance=serde_json::to_value(&view.acceptance)?;
    println!("Acceptance: {} · required: {} · blocking findings: {} · not verified: {} · source freshness: {:?}. PASS is limited to the recorded scope; it is not security certification.", acceptance["status"].as_str().unwrap_or_default(), view.acceptance.required, view.acceptance.blocking_findings, view.acceptance.not_verified, view.acceptance.source_freshness);
    for reason in &view.acceptance.reasons { println!("Reason: {reason}"); }
    for d in &view.decisions {
        println!("Decision {} · action: {} · reportId: {} · findingId: {}\n  actor: {} · acceptedAt: {}\n  reason: {}\n  policyDigest: {} · scopeDigest: {}", d.decision_id, d.action, d.report_id, d.finding_id, d.actor, d.accepted_at, d.reason, d.policy_digest, d.scope_digest);
    }
    if let Some(submission)=&view.submission {println!("Submission: {}",serde_json::to_string(submission)?);}
    if let Some(revision)=view.expected_revision {println!("expectedRevision: {revision}");}
    Ok(())
}

fn remember_current_policy(repository: &dyn StandardRepository) -> Result<()> {
    let cwd = std::env::current_dir()?;
    if !cwd.ancestors().any(|directory| {
        directory.join(".speclink.yaml").symlink_metadata().is_ok()
            || directory.join("openspec").symlink_metadata().is_ok()
    }) {
        return Ok(());
    }
    let location = target_location(&TargetArgs {
        root: None,
        change: None,
        json: false,
    })?;
    if location.remote.is_none() {
        let text = location.reader.read_optional_text(
            &format!("{}/config.yaml", location.spec_dir),
            64 * 1024,
            &|| None,
        )?;
        let policy = speclink_core::config::WorkflowConfig::from_text(text.as_deref())
            .map_err(|_| anyhow::anyhow!("invalid_config: workflow policy could not be parsed"))?
            .security();
        repository.remember_policy(
            &format!("local:{}", location.reader.root().to_string_lossy()),
            &policy,
        )?;
    }
    Ok(())
}

struct RemoteTarget {
    url: String,
    origin: String,
    project: String,
    repo: Option<String>,
}
struct TargetLocation {
    reader: speclink_fs::security::scan::AuthorizedProject,
    spec_dir: String,
    remote: Option<RemoteTarget>,
}
impl TargetLocation {
    fn remote_target(
        &self,
        remote: &RemoteTarget,
        repo: String,
        change_name: Option<String>,
    ) -> ReviewTarget {
        ReviewTarget {
            proposal_project_key:None,
            binding: ReviewBinding {
                origin: Some(remote.origin.clone()),
                project: Some(remote.project.clone()),
                repo: Some(repo),
                checkout: Some(self.reader.root().to_string_lossy().into_owned()),
            },
            spec_dir: self.spec_dir.clone(),
            change_name,
            project_name: remote.project.clone(),
        }
    }
    fn review_target(
        &self,
        prepared: &dyn PreparedRepository,
        change_name: Option<String>,
    ) -> Result<ReviewTarget> {
        if let Some(remote) = &self.remote {
            let root = self.reader.root().to_string_lossy();
            let binding=prepared.find_binding(&remote.origin,&remote.project,remote.repo.as_deref(),Some(&root),change_name.as_deref())?.ok_or_else(||anyhow::anyhow!("missing_preparation: explicitly run security prepare for this remote scope and checkout"))?;
            let header=prepared.load_policy(&binding.project_key(),change_name.as_deref())?;
            let mut target=self.remote_target(remote,binding.repo.unwrap(),change_name);
            target.proposal_project_key=header.and_then(|h|h.proposal_project_key);
            Ok(target)
        } else {
            ReviewTarget::local(self.reader.root(), &self.spec_dir, change_name)
        }
    }
}
fn target_location(args: &TargetArgs) -> Result<TargetLocation> {
    let start = match &args.root {
        Some(root) => root.clone(),
        None => std::env::current_dir()?,
    };
    let selected = if args.root.is_some() {
        start
    } else {
        let mut selected = None;
        for directory in start.ancestors() {
            if directory.join(".speclink.yaml").symlink_metadata().is_ok()
                || directory.join("openspec").symlink_metadata().is_ok()
            {
                selected = Some(directory.to_path_buf());
                break;
            }
        }
        selected.unwrap_or(start)
    };
    let reader = speclink_fs::security::scan::AuthorizedProject::open(&selected)?;
    let text = reader
        .read_optional_text(".speclink.yaml", 64 * 1024, &|| None)?
        .unwrap_or_default();
    let config: speclink_core::config::AppConfig = if text.trim().is_empty() {
        Default::default()
    } else {
        serde_yaml::from_str(&text)
            .map_err(|_| anyhow::anyhow!("invalid_config: .speclink.yaml could not be parsed"))?
    };
    let spec_dir = config.spec_dir.unwrap_or_else(|| "openspec".into());
    let remote = if let Some(config) = config.remote {
        let url = std::env::var("SPECLINK_STORE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or(config.url)
            .ok_or_else(|| anyhow::anyhow!("remote.url is required"))?;
        anyhow::ensure!(
            (url.starts_with("http://") || url.starts_with("https://"))
                && !url.contains(['@', '?', '#'])
                && !url.chars().any(char::is_control),
            "invalid_config: remote URL"
        );
        let url = url.trim_end_matches('/').to_owned();
        let origin = speclink_remote::auth::origin_of(&url);
        let project = url
            .rsplit_once("/projects/")
            .map(|(_, p)| p.to_owned())
            .unwrap_or_else(|| url.clone());
        Some(RemoteTarget {
            url,
            origin,
            project,
            repo: config.repo,
        })
    } else {
        None
    };
    Ok(TargetLocation {
        reader,
        spec_dir,
        remote,
    })
}
fn remote_preparation(
    location: &TargetLocation,
    remote: &RemoteTarget,
    change: Option<String>,
    budget: &ReviewBudget,
) -> Result<(ReviewTarget, security::PreparationInput)> {
    use speclink_remote::{auth, client::Client};
    let fetched = auth::with_resolved_credential(&remote.origin, |bearer| {
        let binding = Client::new(&remote.url, bearer, remote.repo.as_deref()).handshake()?;
        if !binding.capabilities.context_snapshots || !binding.capabilities.security_review {
            return Err(speclink_remote::RemoteError {
                message: "unsupported_version: server does not provide context snapshots".into(),
                reason: Some("unsupported_version".into()),
                status: None,
                evidence: None,
            });
        }
        let repo = if binding.repo.key.is_empty() {
            binding.repo.id
        } else {
            binding.repo.key
        };
        let client = Client::new(&remote.url, bearer, Some(&repo));
        let snapshot = client.security_context_snapshot(
            &speclink_protocol::context::ContextSnapshotRequest {
                change: change.clone(),
                flow: Some("verify".into()),
            },
            &|| budget.stop_reason().is_some(),
        )?;
        Ok((repo, binding.security_project_key, snapshot))
    })
    .map_err(|error| match error {
        auth::CredentialError::NotLoggedIn => {
            anyhow::anyhow!("not_logged_in: run speclink auth login for the configured origin")
        }
        auth::CredentialError::Remote(error) => anyhow::anyhow!(
            "remote_prepare_failed: {}",
            error
                .reason
                .unwrap_or_else(|| "remote_request_failed".into())
        ),
        auth::CredentialError::Rotation(_) => {
            anyhow::anyhow!("reauthentication_required: run speclink auth login")
        }
    })?;
    let (repo, canonical_key, snapshot) = fetched.value;
    let mut target = location.remote_target(remote, repo, change);
    target.proposal_project_key=canonical_key;
    let input = security::preparation_from_context(&target, snapshot)?;
    Ok((target, input))
}

fn batch_exit_code(status: &str) -> u8 {
    match status {
        "installed" => 0,
        "cancelled" => 130,
        _ => 1,
    }
}
#[cfg(test)]
mod bundle_tests {
    use super::*;
    #[test]
    fn partial_failed_and_cancelled_imports_keep_their_cli_exit_contract() {
        for (status, code) in [
            ("installed", 0),
            ("partial", 1),
            ("failed", 1),
            ("cancelled", 130),
        ] {
            assert_eq!(batch_exit_code(status), code);
        }
    }
}

fn remote_call<T>(location:&TargetLocation,call:impl Fn(&speclink_remote::client::Client)->std::result::Result<T,speclink_remote::RemoteError>)->Result<T> {
    let remote=location.remote.as_ref().ok_or_else(||anyhow::anyhow!("binding_mismatch"))?;
    let authenticated=speclink_remote::auth::with_resolved_credential(&remote.origin,|bearer|call(&speclink_remote::client::Client::new(&remote.url,bearer,remote.repo.as_deref()))).map_err(|e|match e {speclink_remote::auth::CredentialError::NotLoggedIn=>anyhow::anyhow!("not_logged_in: run speclink auth login"),speclink_remote::auth::CredentialError::Remote(e)=>anyhow::Error::new(e),speclink_remote::auth::CredentialError::Rotation(_)=>anyhow::anyhow!("reauthentication_required: run speclink auth login")})?;
    Ok(authenticated.value)
}
fn remote_context(location:&TargetLocation,application_data:&std::path::Path,data:&dyn StandardRepository,change:&str,basic:Option<&speclink_core::security::report::Report>,cancelled:Arc<AtomicBool>)->Result<(speclink_core::security::report::DeepReportContext,Option<String>)> {
    let prepared=speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?;
    let target=location.review_target(&prepared,Some(change.into()))?;
    let snapshot=prepared.load(&target.binding.project_key(),Some(change))?.ok_or_else(||anyhow::anyhow!("missing_preparation"))?;
    security::prepared_submission_context(&target,&location.reader,data,&snapshot,basic,None,&ReviewBudget::new(cancelled))
}
fn remote_review_command(command:ReviewCommand,location:&TargetLocation,application_data:&std::path::Path,data:Option<&dyn StandardRepository>,cancelled:Arc<AtomicBool>)->Result<()> {
    use speclink_protocol::security::*;
    use speclink_core::security::report::*;
    use std::io::Read;
    let (change,json)=match &command {ReviewCommand::Show{change,json}|ReviewCommand::AddRound{change,json,..}|ReviewCommand::Accept{change,json,..}=>(change,*json)};
    let view=remote_call(location,|client|client.security_review_show(change))?;
    let device=speclink_fs::security::reports::FsSecurityReports::new(application_data)?;
    match command {
        ReviewCommand::Show{change,..}=> {
            let prepared=speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?;
            let computed=(||->Result<_>{
                let data=data.ok_or_else(||anyhow::anyhow!("standard_data_unavailable"))?;
                let target=location.review_target(&prepared,Some(change.clone()))?;
                let basic=device.latest(&target.report_project_key(),Some(&change))?;
                let (context,id)=remote_context(location,application_data,data,&change,basic.as_ref(),cancelled)?;
                let mut submission=security::Submission::from(&context);submission.builtin_report_id=id;
                let frame=security::source_attestation(&context,data)?;
                let mut checked=remote_call(location,|client|client.security_review_check(&change,&SecurityCheckRequest{security_source_attestation:frame.clone()}))?;
                checked.submission=Some(serde_json::from_value(serde_json::to_value(submission)?)?);Ok(checked)
            })();
            let result=match computed {Ok(v)=>v,Err(e)=>{let mut v=view;v.acceptance.reasons.push(e.to_string().split(':').next().unwrap_or("source_freshness_unverified").into());v}};
            print_security_review(&result,json)
        }
        ReviewCommand::AddRound{change,..}=> {
            let data=data.ok_or_else(||anyhow::anyhow!("standard_data_unavailable"))?;
            let mut bytes=Vec::new();std::io::stdin().take(4*1024*1024+1).read_to_end(&mut bytes)?;anyhow::ensure!(bytes.len()<=4*1024*1024,"size_limit: security report input");
            let input:ReviewRoundInput=serde_json::from_slice(&bytes).map_err(|_|anyhow::anyhow!("invalid_report: submission JSON"))?;
            let prepared=speclink_fs::security::prepared::FsPreparedReviews::new(application_data)?;
            let target=location.review_target(&prepared,Some(change.clone()))?;
            let (report,basic,attestation)=match input {
              ReviewRoundInput::Legacy(legacy) => {
                let basic=device.get(&legacy.legacy_import_report_id)?.ok_or_else(||anyhow::anyhow!("report_not_found"))?;
                anyhow::ensure!(basic.project_key()==target.binding.project_key()&&basic.change_name()==Some(change.as_str()),"binding_mismatch");
                (SecurityRoundPayload::Legacy(SecurityLegacyInput{legacy_import_report_id:legacy.legacy_import_report_id,legacy_project_key:basic.project_key().into()}),basic,None)
              }
              ReviewRoundInput::Deep(mut input) => {
                let id=input.builtin_report_id.as_deref().ok_or_else(||anyhow::anyhow!("missing_security_review"))?;
                let basic=if let Some(b)=device.get(id)?{b}else{let document=remote_call(location,|client|client.security_change_report(&change,id))?;Report::from_json(&serde_json::to_vec(&document)?)?};
                let (context,current)=remote_context(location,application_data,data,&change,Some(&basic),cancelled)?;
                anyhow::ensure!(current.as_deref()==Some(id),"security_report_stale: run current built-in review first");
                let frame=security::source_attestation(&context,data)?;
                // Validate and redact before crossing the HTTP boundary; server repeats validation.
                let checked=DeepReport::new(input.clone(),context)?;
                input.findings=checked.findings().to_vec();
                input.checks=checked.checks().to_vec();
                (SecurityRoundPayload::Deep(serde_json::from_slice(&serde_json::to_vec(&input)?)?),basic,Some(frame))
              }
            };
            let basic_document:SecurityBuiltinDocument=serde_json::from_value(serde_json::to_value(&basic)?)?;
            let receipt=remote_call(location,|client|client.security_review_submit(&change,&SecurityRoundRequest{project_key:target.report_project_key(),expected_revision:view.expected_revision.unwrap_or(0),report:report.clone(),builtin_report:Some(basic_document.clone()),security_source_attestation:attestation.clone()}))?;
            if let Err(e)=device.record_publication(&basic,&change,&receipt.report_id){eprintln!("publication_cache_unavailable: {}",e.to_string().split(':').next().unwrap_or("io_error"));}
            if json {crate::common::print_json(&receipt)}else{println!("Saved security review {} · {} · complete: {}",receipt.report_id,receipt.relative_path,receipt.complete);Ok(())}
        }
        ReviewCommand::Accept{change,..}=> {
            let data=data.ok_or_else(||anyhow::anyhow!("standard_data_unavailable"))?;
            let mut bytes=Vec::new();std::io::stdin().take(16*1024+1).read_to_end(&mut bytes)?;anyhow::ensure!(bytes.len()<=16*1024,"size_limit: security acceptance input");
            let input:AcceptRiskInput=serde_json::from_slice(&bytes).map_err(|_|anyhow::anyhow!("invalid_security_decision: JSON"))?;input.validate()?;
            let (context,_)=remote_context(location,application_data,data,&change,None,cancelled)?;
            let frame=security::source_attestation(&context,data)?;
            let receipt=remote_call(location,|client|client.security_review_accept(&change,&SecurityAcceptRequest{expected_revision:view.expected_revision.unwrap_or(0),report_id:input.report_id.clone(),finding_id:input.finding_id.clone(),reason:input.reason.clone(),security_source_attestation:frame.clone()}))?;
            if json {crate::common::print_json(&receipt)}else{println!("Accepted security risk {} · report {} · finding {}",receipt.decision_id,receipt.report_id,receipt.finding_id);Ok(())}
        }
    }
}

pub(crate) fn remote_security_gate(change:&str)->Result<speclink_protocol::security::SecurityGateRequest> {
    let location=target_location(&TargetArgs{root:None,change:Some(change.into()),json:false})?;
    let application_data=speclink_host::context::security_data_dir()?;
    let data=crate::common::security_repository(&application_data)?;
    let (context,_)=remote_context(&location,&application_data,&data,change,None,Arc::new(AtomicBool::new(false)))?;
    let frame=security::source_attestation(&context,&data)?;
    let view=remote_call(&location,|client|client.security_review_show(change))?;
    Ok(speclink_protocol::security::SecurityGateRequest{expected_revision:view.expected_revision,security_source_attestation:Some(frame)})
}
