import {useEffect,useState} from 'react';
import type {CheckRecordEntry,CheckRecordsAdapter} from '../checkRecords';
import {useI18n} from '../i18n';
import {TabsContent} from './ui/tabs';
import {Markdown} from './Markdown';
import {DocSkeleton} from './skeletons';

export const checkTab=(entry:CheckRecordEntry)=>`check:${entry.kind}:${entry.recordId}`;
const errorText=(error:unknown)=>error instanceof Error?error.message:String(error);
const errorLabel=(message:string,t:(key:string)=>string)=>message==='reauthentication_required'?t('checks.reauthenticate'):message;
const startedAt=(entry:CheckRecordEntry)=>{
 const segments=entry.relativePath.split('/');const name=segments[segments.length-1]??'';
 const match=/^(\d{14})[0-7][0-9A-HJKMNP-TV-Z]{25}\.md$/.exec(name);
 if(!match)return entry.createdAt;
 const value=match[1];
 return `${value.slice(0,4)}-${value.slice(4,6)}-${value.slice(6,8)} ${value.slice(8,10)}:${value.slice(10,12)}:${value.slice(12,14)} UTC`;
};
const startedOrder=(entry:CheckRecordEntry)=>{
 const segments=entry.relativePath.split('/');const name=segments[segments.length-1]??'';
 const match=/^(\d{14})[0-7][0-9A-HJKMNP-TV-Z]{25}\.md$/.exec(name);
 if(!match)return Date.parse(entry.createdAt)||0;
 const value=match[1];
 return Date.UTC(Number(value.slice(0,4)),Number(value.slice(4,6))-1,Number(value.slice(6,8)),Number(value.slice(8,10)),Number(value.slice(10,12)),Number(value.slice(12,14)));
};

/** Every response remains bound to the adapter, proposal and refresh that requested it. */
export function useCheckRecords(adapter:CheckRecordsAdapter|undefined,change:string|null,tab:string,gen:number,enabled=true){
 type List={adapter:CheckRecordsAdapter;change:string;gen:number;entries:CheckRecordEntry[];error?:string};
 type Body={adapter:CheckRecordsAdapter;change:string;gen:number;tab:string;content:string|null;error?:string};
 const [list,setList]=useState<List>();const [body,setBody]=useState<Body>();const [retrySeq,setRetrySeq]=useState(0);
 useEffect(()=>{
  if(!adapter||!change||!enabled)return;let active=true;
  void adapter.list(change).then(entries=>{if(active)setList({adapter,change,gen,entries:[...entries].sort((a,b)=>startedOrder(b)-startedOrder(a)||a.recordId.localeCompare(b.recordId))})},error=>{if(active)setList(previous=>({adapter,change,gen,entries:previous?.adapter===adapter&&previous.change===change?previous.entries:[],error:errorText(error)}))});
  return()=>{active=false};
 },[adapter,change,gen,retrySeq,enabled]);
 const current=list?.adapter===adapter&&list?.change===change?list:undefined;
 const entry=current?.entries.find(entry=>checkTab(entry)===tab);
 useEffect(()=>{
  if(!adapter||!change||!entry||entry.outcome==='omitted')return;let active=true;
  void adapter.read(change,entry).then(content=>{if(active)setBody({adapter,change,gen,tab,content})},error=>{if(active)setBody(previous=>({adapter,change,gen,tab,content:previous?.adapter===adapter&&previous.change===change&&previous.tab===tab?previous.content:null,error:errorText(error)}))});
  return()=>{active=false};
 },[adapter,change,entry,gen,tab,retrySeq]);
 const document=entry?.outcome==='omitted'?undefined:body?.adapter===adapter&&body?.change===change&&body?.tab===tab?body:undefined;
 const retry=()=>{setList(previous=>previous?{...previous,gen:-1,error:undefined}:previous);setBody(undefined);setRetrySeq(value=>value+1)};
 return {enabled:!!adapter,entries:current?.entries??[],listError:current?.error,loading:!!adapter&&enabled&&(!current||current.gen!==gen),entry,content:entry?document?.content:undefined,error:entry?document?.error:undefined,retry};
}
export function QualityCheckRecordList({state,kind,onSelect,id,hidden}:{state:ReturnType<typeof useCheckRecords>;kind:'review'|'verify';onSelect:(entry:CheckRecordEntry)=>void;id:string;hidden:boolean}){
 const {t}=useI18n();const title=t(kind==='review'?'checks.reviewRecords':'checks.verifyRecords');
 const entries=state.entries.filter(entry=>entry.kind===kind);
 return <section id={id} aria-label={title} hidden={hidden} className="mt-1 w-full basis-full space-y-1 pl-1">
  {state.loading&&<p className="text-xs text-muted-foreground">{t('checks.loading')}</p>}
  {!state.loading&&state.listError&&<div role="alert" className="flex items-center gap-2 text-xs text-destructive"><span>{errorLabel(state.listError,t)}</span><button type="button" className="underline" onClick={state.retry}>{t('checks.retry')}</button></div>}
  {!state.loading&&!state.listError&&entries.length===0&&<p className="text-xs text-muted-foreground">{t('checks.empty')}</p>}
  {!state.loading&&!state.listError&&entries.map(entry=><button key={checkTab(entry)} type="button" className="flex w-full items-center justify-between gap-2 rounded px-2 py-1 text-left text-xs hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" onClick={()=>onSelect(entry)}>
   <span className="min-w-0 truncate font-mono">{startedAt(entry)} · {entry.recordId}</span><span className="shrink-0 text-muted-foreground">{t(`checks.outcome.${entry.outcome}`)}</span>
  </button>)}
 </section>;
}
export function CheckRecordBody({state,tab}:{state:ReturnType<typeof useCheckRecords>;tab:string}){
 const {t}=useI18n();if(!tab.startsWith('check:'))return null;
 return <TabsContent value={tab}>{state.error&&<p role="alert" className="text-sm text-destructive">{errorLabel(state.error,t)}</p>}{!state.entry?<p className="text-sm text-muted-foreground">{t('checks.empty')}</p>:state.entry.outcome==='omitted'?<p className="text-sm text-muted-foreground">{t('checks.omitted')}</p>:state.content===undefined?<DocSkeleton/>:<Markdown content={state.content} empty={t('checks.empty')}/>}</TabsContent>;
}
