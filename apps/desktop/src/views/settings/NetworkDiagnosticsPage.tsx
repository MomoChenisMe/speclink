import {useEffect,useState} from "react";
import {Button,Input,SettingsCard,SettingsChoice,SettingsPageHeader,SettingsRow,Switch,useI18n} from "@speclink/ui";
import {readRuntimePreferences,writeRuntimePreferences,exportDiagnostics,probeUpdate,type RuntimePreferences,type RuntimeSnapshot,type ProbeResult} from "../../adapter/runtime";
const DEFAULTS:RuntimePreferences={formatVersion:1,debugEnabled:false,proxy:{mode:"system",url:null}};
const codeOf=(error:unknown)=>error && typeof error==="object" && "code" in error ? String(error.code) : "runtime_unavailable";
export function NetworkDiagnosticsPage() {
  const {t}=useI18n();
  const [snapshot,setSnapshot]=useState<RuntimeSnapshot|null>(null);
  const [draft,setDraft]=useState<RuntimePreferences>(DEFAULTS);
  const [error,setError]=useState<string|null>(null);
  const [busy,setBusy]=useState(false);
  const [exportPath,setExportPath]=useState<string|null>(null);
  const [probe,setProbe]=useState<ProbeResult|null>(null);
  useEffect(()=>{
    let disposed=false;
    readRuntimePreferences().then(value=>{
      if(disposed)return;
      setSnapshot(value);setDraft({formatVersion:1,debugEnabled:value.debugEnabled,proxy:value.proxy});
    }).catch(error=>{if(!disposed)setError(codeOf(error));});
    return ()=>{disposed=true;};
  },[]);
  useEffect(()=>{
    if(!snapshot?.debugEnabled)return;
    let disposed=false;
    const timer=setInterval(()=>void readRuntimePreferences().then(value=>{
      if(!disposed)setSnapshot(value);
    }).catch(()=>{}),2000);
    return ()=>{disposed=true;clearInterval(timer);};
  },[snapshot?.debugEnabled]);
  async function persist(preferences:RuntimePreferences) {
    setBusy(true);setError(null);
    try {const value=await writeRuntimePreferences(preferences);setSnapshot(value);setDraft(preferences);}
    catch(error){setError(codeOf(error));} finally{setBusy(false);}
  }
  async function exportLog() {
    setBusy(true);setError(null);setExportPath(null);
    try {const receipt=await exportDiagnostics();if(receipt)setExportPath(receipt.path);}
    catch(error){setError(codeOf(error));} finally{setBusy(false);}
  }
  async function runProbe() {
    setBusy(true);setError(null);setProbe(null);
    try {setProbe(await probeUpdate());} catch(error){setError(codeOf(error));} finally{setBusy(false);}
  }
  return <>
    <SettingsPageHeader title={t("network.title")} description={t("network.help")} actions={<Button size="sm" disabled={busy||!snapshot||!!snapshot.readError} onClick={()=>void persist(draft)}>{t("settings.save")}</Button>}/>
    {(error||snapshot?.readError)&&<p role="alert" className="text-sm text-destructive">{error??snapshot?.readError}</p>}
    {snapshot?.readError&&<Button variant="secondary" disabled={busy} onClick={()=>void persist(DEFAULTS)}>{t("network.reset")}</Button>}
    <SettingsCard title={t("network.proxyTitle")}>
      <SettingsRow label={t("network.proxyMode")} description={t("network.nextJob")} control={<SettingsChoice ariaLabel={t("network.proxyMode")} value={draft.proxy.mode} disabled={busy||!snapshot||!!snapshot.readError} options={(["system","manual","direct"] as const).map(value=>({value,label:t(`network.mode.${value}`)}))} onChange={value=>setDraft({...draft,proxy:{mode:value as RuntimePreferences["proxy"]["mode"],url:value==="manual"?(draft.proxy.url??""):null}})}/>}/>
      {draft.proxy.mode==="manual"&&<SettingsRow label={t("network.proxyUrl")} control={<Input aria-label={t("network.proxyUrl")} type="url" autoComplete="off" placeholder="http://proxy.example:8080" value={draft.proxy.url??""} disabled={busy||!!snapshot?.readError} onChange={event=>setDraft({...draft,proxy:{mode:"manual",url:event.target.value}})}/>}/>}
      <p className="px-4 pb-4 text-xs text-muted-foreground">{t("network.support")}</p>
    </SettingsCard>
    <SettingsCard title={t("network.diagnostics")}>
      <SettingsRow label="Debug mode" description={t("network.debugHelp")} control={<Switch aria-label="Debug mode" checked={draft.debugEnabled} disabled={busy||!snapshot||!!snapshot.readError} onCheckedChange={checked=>setDraft({...draft,debugEnabled:checked})}/>}/>
      <SettingsRow label={t("network.logStatus")} control={<span role="status" className="text-sm">{snapshot?t(`network.status.${snapshot.diagnosticsStatus}`):t("network.loading")}</span>}/>
      <div className="flex flex-wrap gap-2 px-4 pb-4">
        <Button variant="secondary" size="sm" disabled={busy} onClick={()=>void exportLog()}>{t("network.export")}</Button>
        {snapshot?.debugEnabled&&!snapshot.readError&&<Button variant="secondary" size="sm" disabled={busy} onClick={()=>void runProbe()}>{t("network.probe")}</Button>}
      </div>
      {exportPath&&<p role="status" className="px-4 pb-4 text-sm break-all">{t("network.exported")}: {exportPath}</p>}
      {probe&&<div className="space-y-2 px-4 pb-4 text-sm"><p>{t("network.probeHelp")}</p>{probe.results.map((result,index)=><p key={index}>{result.destination} · {result.httpStatus??"—"} · {result.durationMs} ms{result.errorCode?` · ${result.errorCode==="system-no-static-route"?t("network.systemDirect"):result.errorCode}`:""}</p>)}</div>}
    </SettingsCard>
  </>;
}
