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
        #[arg(long, value_enum, default_value = "json")]
        format: Format,
    },
    /// Manage official standard data stored on this device
    Data {
        #[command(subcommand)]
        command: DataCommand,
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
    let repository = speclink_fs::security::FsSecurityData::new(&application_data)?;
    let job = DataJob::new(cancelled.clone());
    let progress = |p: speclink_core::security::JobProgress| {
        eprintln!(
            "{}: {} bytes, {} files",
            p.phase, p.completed_bytes, p.completed_files
        );
    };
    match args.command {
        SecurityCommand::Rules { .. } => unreachable!(),
        SecurityCommand::Report { id, format } => {
            let reports =
                speclink_fs::security::reports::FsSecurityReports::new(&application_data)?;
            let text = security::export_report(
                &reports,
                &id,
                match format {
                    Format::Json => security::ReportFormat::Json,
                    Format::Markdown => security::ReportFormat::Markdown,
                },
            )?;
            println!("{text}");
            Ok(())
        }
        SecurityCommand::Run(args) => {
            let prepared =
                speclink_fs::security::prepared::FsPreparedReviews::new(&application_data)?;
            let location = target_location(&args)?;
            let target = location.review_target(&prepared, args.change.clone())?;
            let reports =
                speclink_fs::security::reports::FsSecurityReports::new(&application_data)?;
            let outcome = security::run_review(
                &target,
                Some(&location.reader),
                &repository,
                &prepared,
                &reports,
                &ReviewBudget::new(cancelled),
                &progress,
            )?;
            if args.json {
                crate::common::print_json(&outcome.report)?;
            } else {
                println!("{}", outcome.report.summary());
            }
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
        SecurityCommand::Data { command } => match command {
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
            Ok(self.remote_target(remote, binding.repo.unwrap(), change_name))
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
        Ok((repo, snapshot))
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
    let (repo, snapshot) = fetched.value;
    let target = location.remote_target(remote, repo, change);
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
