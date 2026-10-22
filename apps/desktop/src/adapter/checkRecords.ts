import type {CheckKind,CheckRecordEntry,CheckRecordsAdapter} from '@speclink/ui';
import type {InvokeFn} from '../session';

type Receipt={createdAt:string;content:string;contentDigest:string};
type Scope={root:string}|{connectionId:string;project:string;repo:string};
/** Each adapter captures one workspace; no shared current-project or document cache. */
export function createCheckRecordsAdapter(scope:Scope,invoke:InvokeFn,includeSecurity=true):CheckRecordsAdapter{
 const receipts=new Map<string,Receipt>();
 const remote='connectionId' in scope;
 const history=remote?'remote_check_history':'check_history';
 return {
  async list(change){
   const kinds:CheckKind[]=includeSecurity?['review','verify','security']:['review','verify'];
   const lists=await Promise.all(kinds.map(kind=>invoke<{records:CheckRecordEntry[];acceptanceReceipt?:Receipt}>(history,{...scope,change,kind})));
   const receipt=lists.find(list=>list.acceptanceReceipt)?.acceptanceReceipt;
   if(receipt)receipts.set(change,receipt);else receipts.delete(change);
   return [...lists.flatMap(list=>list.records),...(receipt?[{recordId:'acceptance',kind:'security' as const,createdAt:receipt.createdAt,outcome:'passed',relativePath:'checks/acceptance.json',contentDigest:receipt.contentDigest}]:[])];
  },
  async read(change,entry){
   if(entry.kind==='security'&&entry.recordId==='acceptance')return receipts.get(change)?.content??null;
   if(entry.kind==='security')return invoke<string>(remote?'remote_security_record':'security_record',{...scope,change,id:entry.recordId});
   const record=await invoke<{content:string}>(history,{...scope,change,kind:entry.kind,id:entry.recordId});return record.content;
  },
 };
}
