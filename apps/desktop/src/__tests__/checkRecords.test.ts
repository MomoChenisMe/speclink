import {expect,it,vi} from 'vitest';
import {createTauriDataSource} from '../adapter/tauriDataSource';
import {createRemoteDataSource} from '../adapter/remoteDataSource';

it('本機與封存紀錄只使用原 root、提案與 index ID，不把顯示路徑當任意檔案讀取',async()=>{
 const ipc=vi.fn().mockImplementation(async(command:string,args:Record<string,unknown>)=>command==='security_record'?'# 遮罩安全紀錄':args.id?{content:'# 保存紀錄'}:{records:[{recordId:args.kind,kind:args.kind,createdAt:'2026-10-20',outcome:'passed',relativePath:'checks/safe.md',contentDigest:'a'.repeat(64)}]});
 const api=createTauriDataSource('/project-a',ipc).checkRecords!;
 const list=await api.list('2026-10-20-demo');expect(list).toHaveLength(3);
 expect(ipc).toHaveBeenCalledWith('check_history',{root:'/project-a',change:'2026-10-20-demo',kind:'security'});
 await api.read('2026-10-20-demo',{...list[0],relativePath:'../../not-used'});expect(ipc).toHaveBeenCalledWith('check_history',{root:'/project-a',change:'2026-10-20-demo',kind:'review',id:'review'});
 expect(await api.read('2026-10-20-demo',list[2])).toBe('# 遮罩安全紀錄');expect(ipc).toHaveBeenCalledWith('security_record',{root:'/project-a',change:'2026-10-20-demo',id:'security'});
});
it('Remote reader 不需 checkout 即可查紀錄，離線失敗不改用本機來源',async()=>{
 const ipc=vi.fn().mockResolvedValue({records:[]});const api=createRemoteDataSource('connection-a','project-a','repo-a',ipc,{supported:false,history:true,securityV2:true}).checkRecords!;
 await api.list('demo');expect(ipc).toHaveBeenCalledWith('remote_check_history',{connectionId:'connection-a',project:'project-a',repo:'repo-a',change:'demo',kind:'review'});
 ipc.mockRejectedValue(new Error('offline'));await expect(api.list('demo')).rejects.toThrow('offline');expect(ipc.mock.calls.every(([command])=>command.startsWith('remote_'))).toBe(true);
});
it('不保留完整報告仍可讀封存驗收摘要，摘要不送入報告 ID 端點',async()=>{
 const receipt={createdAt:'2026-10-21',content:'# 安全驗收摘要\n\n不作為新一輪驗收的證據。',contentDigest:'b'.repeat(64)};
 const ipc=vi.fn().mockImplementation(async(_command:string,args:Record<string,unknown>)=>({records:[],...(args.kind==='security'?{acceptanceReceipt:receipt}:{})}));
 const api=createTauriDataSource('/project-a',ipc).checkRecords!;
 const list=await api.list('2026-10-21-demo');expect(list).toHaveLength(1);expect(list[0].relativePath).toBe('checks/acceptance.json');
 expect(await api.read('2026-10-21-demo',list[0])).toBe(receipt.content);
 expect(ipc.mock.calls.every(([command])=>command==='check_history')).toBe(true);
});
