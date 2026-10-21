import {useEffect,useState} from 'react';
import {FileCheck} from 'lucide-react';
import type {CheckRecordEntry,CheckRecordsAdapter} from '../checkRecords';
import {useI18n} from '../i18n';
import {DetailTreeLabel} from './DetailShell';
import {TabsContent,TabsList,TabsTrigger} from './ui/tabs';
import {Markdown} from './Markdown';
import {DocSkeleton} from './skeletons';

export const checkTab=(entry:CheckRecordEntry)=>`check:${entry.kind}:${entry.recordId}`;
const errorText=(error:unknown)=>error instanceof Error?error.message:String(error);
const errorLabel=(message:string,t:(key:string)=>string)=>message==='reauthentication_required'?t('checks.reauthenticate'):message;

/** Every response remains bound to the adapter, proposal and refresh that requested it. */
export function useCheckRecords(adapter:CheckRecordsAdapter|undefined,change:string|null,tab:string,gen:number){
 type List={adapter:CheckRecordsAdapter;change:string;gen:number;entries:CheckRecordEntry[];error?:string};
 type Body={adapter:CheckRecordsAdapter;change:string;gen:number;tab:string;content:string|null;error?:string};
 const [list,setList]=useState<List>();const [body,setBody]=useState<Body>();
 useEffect(()=>{
  if(!adapter||!change)return;let active=true;
  void adapter.list(change).then(entries=>{if(active)setList({adapter,change,gen,entries:[...entries].sort((a,b)=>b.createdAt.localeCompare(a.createdAt)||a.recordId.localeCompare(b.recordId))})},error=>{if(active)setList(previous=>({adapter,change,gen,entries:previous?.adapter===adapter&&previous.change===change?previous.entries:[],error:errorText(error)}))});
  return()=>{active=false};
 },[adapter,change,gen]);
 const current=list?.adapter===adapter&&list?.change===change?list:undefined;
 const entry=current?.entries.find(entry=>checkTab(entry)===tab);
 useEffect(()=>{
  if(!adapter||!change||!entry||entry.outcome==='omitted')return;let active=true;
  void adapter.read(change,entry).then(content=>{if(active)setBody({adapter,change,gen,tab,content})},error=>{if(active)setBody(previous=>({adapter,change,gen,tab,content:previous?.adapter===adapter&&previous.change===change&&previous.tab===tab?previous.content:null,error:errorText(error)}))});
  return()=>{active=false};
 },[adapter,change,entry,gen,tab]);
 const document=entry?.outcome==='omitted'?undefined:body?.adapter===adapter&&body?.change===change&&body?.tab===tab?body:undefined;
 return {enabled:!!adapter,entries:current?.entries??[],listError:current?.error,loading:!!adapter&&(!current||current.gen!==gen),entry,content:entry?document?.content:undefined,error:entry?document?.error:undefined};
}
export function CheckRecordTree({state}:{state:ReturnType<typeof useCheckRecords>}){
 const {t}=useI18n();if(!state.enabled)return null;
 return <><DetailTreeLabel>{t('checks.title')}</DetailTreeLabel>
 {state.listError&&<p role="alert" className="px-3 text-xs text-destructive">{errorLabel(state.listError,t)}</p>}{state.loading&&<p className="px-3 text-xs text-muted-foreground">{t('checks.loading')}</p>}{state.entries.length===0?(!state.loading&&!state.listError&&<p className="px-3 text-xs text-muted-foreground">{t('checks.empty')}</p>):<TabsList aria-label={t('checks.title')}>
  {(['review','verify','security'] as const).flatMap(kind=>state.entries.filter(e=>e.kind===kind).map(entry=><TabsTrigger key={checkTab(entry)} value={checkTab(entry)} title={entry.relativePath}><FileCheck className="h-3.5 w-3.5 shrink-0"/><span className="min-w-0 text-left"><span className="block">{t(entry.recordId==='acceptance'&&kind==='security'?'checks.acceptanceReceipt':`checks.${kind}`)} · {entry.createdAt}</span><span className="block text-xs text-muted-foreground">{t(`checks.outcome.${entry.outcome}`)}</span></span></TabsTrigger>))}
 </TabsList>}</>;
}
export function CheckRecordBody({state,tab}:{state:ReturnType<typeof useCheckRecords>;tab:string}){
 const {t}=useI18n();if(!tab.startsWith('check:'))return null;
 return <TabsContent value={tab}>{state.error&&<p role="alert" className="text-sm text-destructive">{errorLabel(state.error,t)}</p>}{!state.entry?<p className="text-sm text-muted-foreground">{t('checks.empty')}</p>:state.entry.outcome==='omitted'?<p className="text-sm text-muted-foreground">{t('checks.omitted')}</p>:state.content===undefined?<DocSkeleton/>:<Markdown content={state.content} empty={t('checks.empty')}/>}</TabsContent>;
}
