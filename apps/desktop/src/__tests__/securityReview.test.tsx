import {expect,it,vi} from "vitest";
import {createSecurityReviewAdapter} from "../adapter/security";
it("Desktop adapter 綁定開始時根目錄及工作身分，無 Agent 也可呼叫",async()=>{
 const invoke=vi.fn().mockImplementation(async(command:string)=>command==="security_inspect"?{projectKey:"key",enabled:true,latest:null,requiresPreparation:false,selectedStandards:[]}:command==="security_start"?{projectKey:"key",jobId:"job"}:command==="security_wait"?{projectKey:"key",reportId:"report"}:null);
 const api=createSecurityReviewAdapter({kind:"local",root:"/authorized/project"},invoke);
 await api.inspect("demo");const ticket=await api.start("demo");await api.wait(ticket);await api.cancel(ticket);
 expect(invoke).toHaveBeenCalledWith("security_inspect",{target:{kind:"local",root:"/authorized/project"},change:"demo"});
 expect(invoke).toHaveBeenCalledWith("security_start",{target:{kind:"local",root:"/authorized/project"},change:"demo"});
 expect(invoke).toHaveBeenCalledWith("security_cancel",{jobId:"job",projectKey:"key"});
});

it("Remote 未宣告能力時不提供掃描；支援時保留 checkout 身分",async()=>{
 const {createRemoteDataSource}=await import("../adapter/remoteDataSource");
 const ipc=vi.fn().mockResolvedValue(null);
 expect(createRemoteDataSource("conn","project","repo",ipc).securityReview).toBeUndefined();
 const api=createRemoteDataSource("conn","project","repo",ipc,{supported:true,checkoutRoot:"/checkout"}).securityReview!;
 await api.start("demo");expect(ipc).toHaveBeenCalledWith("security_start",{target:{kind:"remote",connectionId:"conn",project:"project",repo:"repo",checkoutRoot:"/checkout"},change:"demo"});
 await api.prepare!("demo");expect(ipc).toHaveBeenCalledWith("security_prepare",{target:{kind:"remote",connectionId:"conn",project:"project",repo:"repo",checkoutRoot:"/checkout"},change:"demo"});
});

it('報告讀取保留提案及專案範圍，不借用切換後的頁面',async()=>{
 const ipc=vi.fn().mockResolvedValue(null)
 const a=createSecurityReviewAdapter({kind:'local',root:'/project-a'},ipc)
 const b=createSecurityReviewAdapter({kind:'local',root:'/project-b'},ipc)
 await a.report('report-a','change-a')
 await b.report('report-b','change-b')
 expect(ipc).toHaveBeenCalledWith('security_report',{target:{kind:'local',root:'/project-a'},change:'change-a',id:'report-a'})
 expect(ipc).toHaveBeenCalledWith('security_report',{target:{kind:'local',root:'/project-b'},change:'change-b',id:'report-b'})
})
