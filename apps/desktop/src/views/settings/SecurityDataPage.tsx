import { useEffect, useRef, useState } from "react";
import { Button, SettingsCard, SettingsPageHeader, SettingsRow, useI18n } from "@speclink/ui";
import { cancelSecurityJob, chooseSecurityPackage, chooseSecurityExport, listSecurityData, startSecurityData, waitSecurityData, watchSecurityProgress, type BatchImportReceipt, type DataExportReceipt, type SecurityDataRequest, type SecurityJobTicket, type SecurityProgress, type StandardDataList } from "../../adapter/security";

import { openExternal } from "./AboutPage";

const standardNames: Record<string,string> = {owasp_top10:"OWASP Top 10",owasp_api_top10:"OWASP API Security Top 10",owasp_asvs:"OWASP ASVS",cwe:"MITRE CWE"};
const errorMessage = (error:unknown) => error && typeof error === "object" && "message" in error ? String(error.message) : String(error);

export function SecurityDataPage() {
  const { t } = useI18n();
  const [data,setData] = useState<StandardDataList|null>(null);
  const [error,setError] = useState<string|null>(null);
  const [receipt,setReceipt] = useState<BatchImportReceipt|DataExportReceipt|null>(null);
  const [busy,setBusy] = useState(false);
  const [request,setRequest] = useState<SecurityDataRequest|null>(null);
  const [progress,setProgress] = useState<SecurityProgress|null>(null);
  const [ticket,setTicket] = useState<SecurityJobTicket|null>(null);
  const active = useRef<SecurityJobTicket|null>(null);
  useEffect(() => {
    let disposed = false;
    let unlisten: (()=>void)|undefined;
    listSecurityData().then((value)=>{if(!disposed)setData(value);}).catch((e)=>{if(!disposed)setError(errorMessage(e));});
    watchSecurityProgress((event)=>{
      if (event.jobId === active.current?.jobId && event.projectKey === active.current.projectKey) setProgress(event);
    }).then((stop)=>{if(disposed)stop();else unlisten=stop;}).catch((e)=>{if(!disposed)setError(errorMessage(e));});
    return ()=>{disposed=true;unlisten?.();};
  },[]);

  async function run(request:SecurityDataRequest) {
    setBusy(true);setRequest(request);setError(null);setProgress(null);setReceipt(null);
    try {
      const next = await startSecurityData(request);
      active.current=next;setTicket(next);
      const result = await waitSecurityData(next.jobId);
      if (result && typeof result === "object") {
        if ("results" in result && "formatVersion" in result && result.formatVersion===2) setReceipt(result as BatchImportReceipt);
        else if (request.action==="export" && "path" in result) setReceipt(result as DataExportReceipt);
      }
      if (request.action === "refresh" && result && typeof result === "object" && "standards" in result) setData(result as StandardDataList);
      else setData(await listSecurityData());
    } catch(e) { setError(errorMessage(e)); }
    finally { active.current=null;setTicket(null);setBusy(false);setRequest(null); }
  }
  async function importPackage() {
    try { const path=await chooseSecurityPackage();if(path)await run({action:"import",path}); }
    catch(e) { setError(errorMessage(e)); }
  }
  async function exportPackage(standardId?:string,version?:string) {
    try {
      const path=await chooseSecurityExport(standardId?`${standardId}-${version}.zip`:"standards.zip");
      if(path)await run(standardId&&version?{action:"export",standardId,version,path}:{action:"export",all:true,path});
    } catch(e){setError(errorMessage(e));}
  }
  const sourceLink = (url:string,label:string) => url.startsWith("https://") ? <a className="underline" href={url} onClick={(event)=>{event.preventDefault();openExternal(url);}}>{label}</a> : <span>{label}</span>;
  return <>
    <SettingsPageHeader title={t("securityData.title")} description={t("securityData.help")} actions={<div className="flex gap-2">
      <Button size="sm" variant="secondary" disabled={busy} onClick={()=>void run({action:"refresh"})}>{t("securityData.refresh")}</Button>
      <Button size="sm" variant="secondary" disabled={busy} onClick={()=>void exportPackage()}>{t("securityData.exportAll")}</Button>
      <Button size="sm" variant="secondary" disabled={busy} onClick={()=>void importPackage()}>{t("securityData.import")}</Button>
    </div>} />
    <p className="text-sm text-muted-foreground">{t("securityData.referenceScope")}</p>
    {error && <p role="alert" className="text-sm text-destructive">{error}</p>}
    {receipt && ("results" in receipt ? <div role={receipt.status==="installed"?"status":"alert"} className="space-y-2 text-sm">
      <p>{t(`securityData.batch.${receipt.status}`)}</p>
      {receipt.results.map((item,index)=><p key={index}>{standardNames[item.standardId]??item.standardId} {item.version} · {t(`securityData.batchItem.${item.status}`)}{item.errorCode?` · ${item.errorCode}`:""}</p>)}
    </div> : <div role="status" className="space-y-2 text-sm">
      <p>{t("securityData.exported")} {receipt.path}</p>
      {receipt.skipped.map((item,index)=><p key={index}>{t("securityData.skipped")} {standardNames[item.standardId]??item.standardId} {item.version} · {item.reason}</p>)}
    </div>)}
    {busy && <div role="status" className="flex items-center gap-3 text-sm">
      <span>{progress ? `${t(`securityData.phase.${progress.phase}`)} · ${progress.completedBytes.toLocaleString()} bytes · ${progress.completedFiles} ${t("securityData.files")}` : t("securityData.working")}</span>
      <Button size="sm" variant="secondary" disabled={!ticket} onClick={()=>{if(ticket)void cancelSecurityJob(ticket.jobId).catch((e)=>setError(errorMessage(e)));}}>{t("securityData.cancel")}</Button>
    </div>}
    {data?.standards.map((standard)=><SettingsCard key={standard.standardId} title={standardNames[standard.standardId]??standard.standardId}>
      {standard.versions.map((version)=><div key={version.version}>
        <SettingsRow label={version.version} description={t(`securityData.status.${version.status}`)} control={<div className="flex flex-wrap gap-2">
          <Button size="sm" variant="secondary" disabled={busy||version.status==="unavailable"} onClick={()=>void run({action:"install",standardId:standard.standardId,version:version.version,reinstall:version.status!=="missing"})}>{t(request?.action==="install" && request.standardId===standard.standardId && request.version===version.version ? "securityData.installing" : version.status==="installed"||version.status==="invalid"?"securityData.reinstall":"securityData.install")}</Button>
          {version.status==="installed"&&<Button size="sm" variant="secondary" disabled={busy} onClick={()=>void exportPackage(standard.standardId,version.version)}>{t("securityData.exportOne")}</Button>}
          {(version.status==="installed"||version.status==="invalid")&&<Button size="sm" variant="secondary" disabled={busy} onClick={()=>void run({action:"remove",standardId:standard.standardId,version:version.version})}>{t("securityData.remove")}</Button>}
        </div>} />
        <div className="space-y-1 px-4 pb-4 text-xs text-muted-foreground break-words">
          <p>{t("securityData.source")}: {sourceLink(version.sourceUrl,version.sourceUrl)}</p>
          <p>{t("securityData.license")}: {sourceLink(version.licenseUrl,version.licenseName)}</p>
          <p>{version.attribution}</p><p>{t("securityData.format")}: {version.sourceFormat}</p>
          {version.error&&<p role="alert">{version.error}</p>}
        </div>
      </div>)}
    </SettingsCard>)}
  </>;
}
