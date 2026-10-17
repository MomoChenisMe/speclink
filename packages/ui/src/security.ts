/** Mirrors the redacted Rust report; no source text or AI execution capability. */
export type SecurityStatus="PASS"|"FINDING"|"NOT_VERIFIED"|"NOT_APPLICABLE"|"DISABLED";
export interface SecurityJobTicket {jobId:string;projectKey:string}
export interface SecurityProgress extends SecurityJobTicket {phase:string;completedBytes:number;totalBytes:number|null;completedFiles:number;message:string}
export interface SecurityReference {standardId:string;version:string;controlId:string;sourceUrl:string;sourceRevision?:string;section?:string}
export interface SecurityFinding {findingId:string;ruleId:string;title:string;severity:string;confidence:string;path:string;line:number;evidence:string;description:string;remediation:string;references:SecurityReference[];confirmationStatus:"needs_review";kind:"potential_risk"|"missing_requirement"}
export interface SecurityCheck {ruleId:string;target:string;status:SecurityStatus;reason:string|null;relatedStandardIds:string[]}
export interface SecurityReport {
 formatVersion:number;reportId:string;projectKey:string;projectName:string;changeName?:string|null;reviewedAt:string;commitSha:string|null;policyDigest?:string;policyRevision?:number|null;preparedAt?:string|null;
 ruleVersion:string;scope:string;executionMode:string;reviewMethods:string[];complete:boolean;configurationState:string;
 counts:{checksSucceeded:number;checksFailed:number;findings:number;notVerified:number;notApplicable:number;disabled:number};
 findings:SecurityFinding[];checks:SecurityCheck[];standards:Array<{standardId:string;version:string|null;level:string|null;installedStatus:string;status:SecurityStatus;reason?:string|null;dataSha256?:string|null;sourceRevision?:string|null}>;
 coverage:{inspectedFiles:number;skippedFiles:number;failedItems:number;bytesRead:number;remainingTargets:string[];reasons:Array<{path:string;status:SecurityStatus;reason:string|null;bytes:number}>};disclaimer:string;
 sourceLicenses:Array<{standardId:string;version:string;sourceUrl:string;licenseName:string;licenseUrl:string;attribution:string;modificationNotice:string}>;sources:Array<{path:string;contentDigest:string}>;
}
export interface SecurityInspection {projectKey:string;enabled:boolean|null;requiresPreparation:boolean;latest:SecurityReport|null;selectedStandards:Array<{standardId:string;version:string;level?:string|null}>}
export interface SecurityReviewAdapter {
 inspect(change:string):Promise<SecurityInspection>;
 start(change:string):Promise<SecurityJobTicket>;
 prepare?(change:string):Promise<SecurityJobTicket>;
 wait(ticket:SecurityJobTicket):Promise<{reportId?:string;projectKey:string}>;
 cancel(ticket:SecurityJobTicket):Promise<unknown>;
 report(id:string):Promise<SecurityReport>;
 exportReport(id:string,format:"json"|"markdown"):Promise<void>;
 copySummary(id:string):Promise<void>;
 subscribe(listener:(progress:SecurityProgress)=>void):Promise<()=>void>;
 reveal?(id:string,path:string,line:number):Promise<void>;
}
