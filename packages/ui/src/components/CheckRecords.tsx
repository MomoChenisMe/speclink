import {forwardRef,useEffect,useRef,useState,type ButtonHTMLAttributes, type ReactNode} from 'react';
import type {CheckRecordEntry,CheckRecordsAdapter} from '../checkRecords';
import {useI18n} from '../i18n';
import {TabsContent} from './ui/tabs';
import {Markdown} from './Markdown';
import {DocSkeleton} from './skeletons';

export const checkTab=(entry:CheckRecordEntry)=>`check:${entry.kind}:${entry.recordId}`;
type QualityHistoryToggleProps=ButtonHTMLAttributes<HTMLButtonElement>&{label:string;expanded:boolean;controlsId:string;onToggle:()=>void;station?:'review'|'verify'};
export const QualityHistoryToggle=forwardRef<HTMLButtonElement,QualityHistoryToggleProps>(function QualityHistoryToggle({label,expanded,controlsId,onToggle,children,station,...buttonProps},ref){
 return <button {...buttonProps} ref={ref} data-review-row={station==='review'?'':undefined} data-verify-row={station==='verify'?'':undefined} type="button" aria-expanded={expanded} aria-controls={controlsId} aria-label={label} onClick={onToggle} className="inline-flex shrink-0 items-center gap-1.5 rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">{children}</button>;
});
const errorText=(error:unknown)=>error instanceof Error?error.message:String(error);
const errorLabel=(message:string,t:(key:string)=>string)=>message==='reauthentication_required'?t('checks.reauthenticate'):message==='missing_check_record'?t('checks.readError.missing_check_record'):message==='integrity_error'?t('checks.readError.integrity_error'):message==='check_record_read_error'?t('checks.readError.check_record_read_error'):message;
const readErrorLabel=(error:NonNullable<CheckRecordEntry['readError']>,t:(key:string)=>string)=>t(`checks.readError.${error}`);
const startedTimestamp=(entry:CheckRecordEntry)=>{
 const segments=entry.relativePath.split('/');const name=segments[segments.length-1]??'';
 const match=/^(\d{14})[0-7][0-9A-HJKMNP-TV-Z]{25}\.md$/.exec(name);
 const value=match?.[1];
 const instant=value?Date.parse(`${value.slice(0,4)}-${value.slice(4,6)}-${value.slice(6,8)}T${value.slice(8,10)}:${value.slice(10,12)}:${value.slice(12,14)}Z`):Date.parse(entry.createdAt);
 return Number.isFinite(instant)?instant:undefined;
};
export const checkRecordStartedAt=(entry:CheckRecordEntry)=>{
 const segments=entry.relativePath.split('/');const name=segments[segments.length-1]??'';
 const match=/^(\d{14})[0-7][0-9A-HJKMNP-TV-Z]{25}\.md$/.exec(name);
 if(!match||startedTimestamp(entry)===undefined)return entry.createdAt;
 const value=match[1];
 return `${value.slice(0,4)}-${value.slice(4,6)}-${value.slice(6,8)} ${value.slice(8,10)}:${value.slice(10,12)}:${value.slice(12,14)} UTC`;
};

/** Every response remains bound to the adapter, proposal and refresh that requested it. */
export function useCheckRecords(adapter:CheckRecordsAdapter|undefined,change:string|null,tab:string,gen:number,enabled=true){
 type List={adapter:CheckRecordsAdapter;change:string;gen:number;entries:CheckRecordEntry[];error?:string};
 type Body={adapter:CheckRecordsAdapter;change:string;gen:number;tab:string;content:string|null;error?:string};
 const [list,setList]=useState<List>();const [body,setBody]=useState<Body>();const [retrySeq,setRetrySeq]=useState(0);const [bodyRetrySeq,setBodyRetrySeq]=useState(0);const handledBodyRetrySeq=useRef(0);const [listVersion,setListVersion]=useState(0);
 useEffect(()=>{
  const forced=bodyRetrySeq>handledBodyRetrySeq.current;
  if(!adapter||!change||(!enabled&&!forced))return;let active=true;
  void adapter.list(change,['review','verify']).then(entries=>{if(active){if(forced)handledBodyRetrySeq.current=bodyRetrySeq;setList({adapter,change,gen,entries:[...entries].sort((a,b)=>{
   const aTime=startedTimestamp(a);const bTime=startedTimestamp(b);
   if(aTime===undefined)return bTime===undefined?a.recordId.localeCompare(b.recordId):1;
   if(bTime===undefined)return -1;
   return bTime-aTime||a.recordId.localeCompare(b.recordId);
  })});setListVersion(version=>version+1)}},error=>{if(active){if(forced){handledBodyRetrySeq.current=bodyRetrySeq;setBody({adapter,change,gen,tab,content:null,error:errorText(error)})}setList(previous=>({adapter,change,gen,entries:previous?.adapter===adapter&&previous.change===change?previous.entries:[],error:errorText(error)}))}});
  return()=>{active=false};
 },[adapter,change,gen,retrySeq,enabled,bodyRetrySeq]);
 const current=list?.adapter===adapter&&list?.change===change?list:undefined;
 const entry=current?.entries.find(entry=>checkTab(entry)===tab);
 useEffect(()=>{
  if(!adapter||!change||!entry||entry.outcome==='omitted'||entry.readError)return;let active=true;
  void adapter.read(change,entry).then(content=>{if(active)setBody(content===null?{adapter,change,gen,tab,content:null,error:'missing_check_record'}:{adapter,change,gen,tab,content})},error=>{if(active)setBody({adapter,change,gen,tab,content:null,error:errorText(error)})});
  return()=>{active=false};
 },[adapter,change,entry,gen,tab,listVersion]);
 const document=entry?.outcome==='omitted'?undefined:body?.adapter===adapter&&body?.change===change&&body?.gen===gen&&body?.tab===tab?body:undefined;
 const retry=()=>{setList(previous=>previous?{...previous,gen:-1,error:undefined}:previous);setBody(undefined);setRetrySeq(value=>value+1)};
 const retryBody=()=>{setList(previous=>previous?{...previous,gen:-1,error:undefined}:previous);setBody(undefined);setBodyRetrySeq(value=>value+1);setRetrySeq(value=>value+1)};
 return {enabled:!!adapter,entries:current?.entries??[],listError:current?.error,loading:!!adapter&&enabled&&(!current||current.gen!==gen),entry,content:entry?document?.content:undefined,error:entry?document?.error:undefined,retry,retryBody};
}
export function QualityCheckRecordList({state,kind,onSelect,id,hidden}:{state:ReturnType<typeof useCheckRecords>;kind:'review'|'verify';onSelect:(entry:CheckRecordEntry)=>void;id:string;hidden:boolean}){
 const {t}=useI18n();const title=t(kind==='review'?'checks.reviewRecords':'checks.verifyRecords');
 const entries=state.entries.filter(entry=>entry.kind===kind);
 return <section id={id} aria-label={title} hidden={hidden} className="mt-1 w-full basis-full space-y-1 pl-1">
  {state.loading&&<p className="text-xs text-muted-foreground">{t('checks.loading')}</p>}
  {!state.loading&&state.listError&&<div role="alert" className="flex items-center gap-2 text-xs text-destructive"><span>{errorLabel(state.listError,t)}</span><button type="button" className="underline" onClick={state.retry}>{t('checks.retry')}</button></div>}
  {!state.loading&&!state.listError&&!state.enabled&&<p className="text-xs text-muted-foreground">{t('checks.unavailable')}</p>}
  {!state.loading&&!state.listError&&state.enabled&&entries.length===0&&<p className="text-xs text-muted-foreground">{t('checks.empty')}</p>}
  {!state.loading&&!state.listError&&state.enabled&&entries.map(entry=><button key={checkTab(entry)} type="button" className="flex w-full items-center justify-between gap-2 rounded px-2 py-1 text-left text-xs hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" onClick={()=>onSelect(entry)}>
   <span className="min-w-0 truncate font-mono">{checkRecordStartedAt(entry)} · {entry.recordId}</span><span className="shrink-0 text-muted-foreground">{t(`checks.outcome.${entry.outcome}`)}{entry.readError&&` · ${readErrorLabel(entry.readError,t)}`}</span>
  </button>)}
 </section>;
}
export function CheckRecordBody({state,tab}:{state:ReturnType<typeof useCheckRecords>;tab:string}){
 const {t}=useI18n();if(!tab.startsWith('check:'))return null;
 return <TabsContent value={tab}>{state.error&&<div role="alert" className="flex items-center gap-2 text-sm text-destructive"><span>{errorLabel(state.error,t)}</span><button type="button" className="underline" onClick={state.retryBody}>{t('checks.retry')}</button></div>}{!state.entry?<p className="text-sm text-muted-foreground">{t('checks.empty')}</p>:state.entry.outcome==='omitted'?<p className="text-sm text-muted-foreground">{t('checks.omitted')}</p>:state.entry.readError?<div role="alert" className="flex items-center gap-2 text-sm text-destructive"><span>{readErrorLabel(state.entry.readError,t)}</span><button type="button" className="underline" onClick={state.retryBody}>{t('checks.retry')}</button></div>:state.error?null:state.content===undefined?<DocSkeleton/>:<Markdown content={state.content} empty={t('checks.empty')}/>}</TabsContent>;
}
