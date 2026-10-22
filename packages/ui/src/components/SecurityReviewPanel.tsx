import {useEffect,useRef,useState} from "react";
import type {SecurityInspection,SecurityJobTicket,SecurityProgress,SecurityReport,SecurityReviewAdapter} from "../security";
import {useI18n} from "../i18n";
import {Button} from "./ui/button";
import {Badge} from "./ui/badge";
import {SettingsCard} from "./ui/settings";

const message=(error:unknown)=>error&&typeof error==="object"&&"message" in error?String(error.message):String(error);
export function SecurityReviewPanel({adapter,change,inspection,onInspection,onOpenDataSettings}:{adapter:SecurityReviewAdapter;change:string;inspection:SecurityInspection;onInspection?:(next:SecurityInspection)=>void;onOpenDataSettings?:()=>void}) {
 const {t}=useI18n();const [report,setReport]=useState<SecurityReport|null>(inspection.latest);const [busy,setBusy]=useState(false);const [error,setError]=useState<string|null>(null);const [progress,setProgress]=useState<SecurityProgress|null>(null);const [visible,setVisible]=useState(100);
 const ticket=useRef<SecurityJobTicket|null>(null);const generation=useRef(0);
 useEffect(()=>{setReport(inspection.latest);setVisible(100);},[inspection.latest?.reportId,inspection.projectKey]);
 useEffect(()=>{
   generation.current+=1;let disposed=false;let stop:(()=>void)|undefined;ticket.current=null;setBusy(false);setProgress(null);setError(null);
   adapter.subscribe((event)=>{if(!disposed&&ticket.current?.jobId===event.jobId&&ticket.current.projectKey===event.projectKey)setProgress(event);}).then((unsubscribe)=>{if(disposed)unsubscribe();else stop=unsubscribe;}).catch((e)=>{if(!disposed)setError(message(e));});
   return ()=>{disposed=true;generation.current+=1;stop?.();};
 },[adapter,change]);
 async function run(mode:"review"|"prepare"|"knowledge"="review") {
   const current=generation.current;setBusy(true);setError(null);setProgress(null);
   try {
     const next=await (mode==="knowledge"&&adapter.prepareKnowledge?adapter.prepareKnowledge(change):mode==="prepare"&&adapter.prepare?adapter.prepare(change):adapter.start(change));
     if(current===generation.current)ticket.current=next;
     const result=await adapter.wait(next);
     if(current!==generation.current)return;
     if(result.projectKey!==inspection.projectKey)throw new Error("binding_mismatch");
     if(result.reportId){const nextReport=await adapter.report(result.reportId,change);if(current===generation.current)setReport(nextReport);}
     else {const fresh=await adapter.inspect(change);if(current===generation.current){onInspection?.(fresh);setReport(fresh.latest);}}
   }catch(e){if(current===generation.current)setError(message(e));}
   finally {if(current===generation.current){ticket.current=null;setBusy(false);}}
 }
 async function action(work:()=>Promise<unknown>){try{await work();setError(null);}catch(e){setError(message(e));}}
 const missing=report?.standards.some((s)=>s.installedStatus==="missing"||s.installedStatus==="invalid");
 return <SettingsCard title={t("securityReview.title")}>
  <div className="space-y-4 p-4 text-sm">
   <p className="text-muted-foreground">{t("securityReview.disclaimer")}</p>
   <p>{inspection.selectedStandards.length?inspection.selectedStandards.map((s)=>`${s.standardId} ${s.version}${s.level?` ${s.level}`:""}`).join(" · "):t("securityReview.unconfigured")}</p>
   {inspection.enabled===false&&<p>{t("securityReview.disabled")}</p>}
   {inspection.deviceLocal&&<p role="status">{t("securityReview.deviceLocal")}</p>}
   {inspection.requiresPreparation&&<p>{t("securityReview.requiresPreparation")}</p>}
   <div className="flex flex-wrap gap-2">
    <Button size="sm" disabled={busy||inspection.enabled!==true||inspection.requiresPreparation} onClick={()=>void run()}>{t("securityReview.run")}</Button>
    {adapter.prepare&&<Button size="sm" variant="secondary" disabled={busy} onClick={()=>void run("prepare")}>{t("securityReview.prepare")}</Button>}
    {adapter.prepareKnowledge&&<Button size="sm" variant="secondary" disabled={busy||inspection.enabled!==true||inspection.requiresPreparation} onClick={()=>void run("knowledge")}>{t("securityReview.prepareKnowledge")}</Button>}
    {busy&&<Button size="sm" variant="secondary" onClick={()=>{const active=ticket.current;if(active)void action(()=>adapter.cancel(active));}}>{t("securityReview.cancel")}</Button>}
    {(missing||report?.configurationState==="unconfigured")&&onOpenDataSettings&&<Button size="sm" variant="secondary" onClick={onOpenDataSettings}>{t("securityReview.dataSettings")}</Button>}
   </div>
   {busy&&<p role="status">{progress?`${t(`securityReview.phase.${progress.phase}`)} · ${progress.completedFiles} · ${progress.completedBytes.toLocaleString()} bytes`:t("securityReview.working")}</p>}
   {error&&<p role="alert" className="text-destructive">{error}</p>}
   {report&&<>
    {report.standards.filter((s)=>s.status!=="DISABLED").map((s)=><p key={s.standardId} className="text-xs">{s.standardId} {s.version} {s.level} · {t(`securityReview.status.${s.status}`)} · {s.installedStatus}{s.reason?` · ${s.reason}`:""}</p>)}
    <div className="flex flex-wrap items-center gap-2"><Badge variant="secondary">{report.complete?t("securityReview.complete"):t("securityReview.partial")}</Badge><code>{report.reportId}</code></div>
    <p className="text-xs text-muted-foreground">{report.reviewedAt} · {report.executionMode} · {report.ruleVersion} · {report.scope}</p>
    {report.preparedAt&&<p className="text-xs text-muted-foreground">{t("securityReview.preparedAt")}: {report.preparedAt} · revision {report.policyRevision??"—"}</p>}
    <p>{t("securityReview.counts").replace("{findings}",String(report.counts.findings)).replace("{unverified}",String(report.counts.notVerified)).replace("{checked}",String(report.counts.checksSucceeded))}</p>
    <div className="flex flex-wrap gap-2"><Button size="sm" variant="secondary" onClick={()=>void action(()=>adapter.exportReport(report.reportId,"json",change))}>{t("securityReview.exportJson")}</Button><Button size="sm" variant="secondary" onClick={()=>void action(()=>adapter.exportReport(report.reportId,"markdown",change))}>{t("securityReview.exportMarkdown")}</Button><Button size="sm" variant="secondary" onClick={()=>void action(()=>adapter.copySummary(report.reportId,change))}>{t("securityReview.copy")}</Button></div>
    <div className="space-y-3">{report.findings.slice(0,visible).map((f)=><div key={f.findingId} className="space-y-2 rounded-md border border-border p-3">
      <div className="flex flex-wrap gap-2"><code>{f.ruleId}</code><Badge variant="secondary">{t(`securityReview.severity.${f.severity}`)}</Badge><span>{t(f.kind==="missing_requirement"?"securityReview.requirementGap":"securityReview.potentialRisk")}</span><span>{t("securityReview.needsReview")} · {t("securityReview.confidence")}: {t(`securityReview.confidence.${f.confidence}`)}</span></div>
      <p>{f.title}</p>{adapter.reveal?<Button type="button" variant="link" size="sm" className="h-auto break-all px-0 font-mono text-xs underline-offset-4" onClick={()=>void action(()=>adapter.reveal!(report.reportId,f.path,f.line,change))}>{f.path}:{f.line}</Button>:<code>{f.path}:{f.line}</code>}
      <pre className="overflow-x-auto whitespace-pre-wrap rounded bg-muted p-2 text-xs">{f.evidence}</pre><p>{f.description}</p><p>{f.remediation}</p>
      {f.references.map((r)=><p key={`${r.standardId}/${r.version}/${r.controlId}`} className="text-xs">{r.standardId} {r.version} · {r.controlId}</p>)}
    </div>)}</div>
    {report.findings.length>visible&&<Button size="sm" variant="secondary" onClick={()=>setVisible((n)=>n+100)}>{t("securityReview.more")}</Button>}
    <details><summary className="cursor-(--cursor-interaction)">{t("securityReview.checks")}</summary><div className="mt-2 space-y-1 text-xs">{report.checks.slice(0,100).map((c,index)=><p key={index}><code>{c.ruleId}</code> · {t(`securityReview.status.${c.status}`)} · {c.target} · {c.reason}</p>)}{report.checks.length>100&&<p>{t("securityReview.exportFull")}</p>}</div></details>
    <p className="text-xs text-muted-foreground">{t("securityReview.coverage")}: {report.coverage.inspectedFiles} · {report.coverage.skippedFiles} · {report.coverage.bytesRead.toLocaleString()} bytes</p>
    {report.coverage.remainingTargets.length>0&&<p>{t("securityReview.remaining")}: {report.coverage.remainingTargets.slice(0,20).join(", ")}</p>}
    {report.sourceLicenses.map((s)=><p key={`${s.standardId}/${s.version}`} className="text-xs text-muted-foreground">{s.standardId} {s.version} · {s.licenseName} · {s.attribution} · {s.licenseUrl}</p>)}
   </>}
  </div>
 </SettingsCard>;
}
