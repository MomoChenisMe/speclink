export type CheckKind='review'|'verify'|'security';
export interface CheckRecordEntry {recordId:string;kind:CheckKind;createdAt:string;outcome:string;relativePath:string;contentDigest:string}
export interface CheckRecordsAdapter {
 list(change:string):Promise<CheckRecordEntry[]>;
 read(change:string,entry:CheckRecordEntry):Promise<string|null>;
}
