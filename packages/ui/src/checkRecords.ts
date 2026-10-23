export type CheckKind='review'|'verify'|'security';
export type CheckRecordReadError='missing_check_record'|'integrity_error'|'check_record_read_error';
export interface CheckRecordEntry {recordId:string;kind:CheckKind;createdAt:string;outcome:string;relativePath:string;contentDigest:string;readError?:CheckRecordReadError}
export interface CheckRecordsAdapter {
 list(change:string,kinds?:CheckKind[]):Promise<CheckRecordEntry[]>;
 read(change:string,entry:CheckRecordEntry):Promise<string|null>;
}
