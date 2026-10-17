import { act,fireEvent,render,screen,waitFor } from "@testing-library/react";
import { expect,it,vi } from "vitest";
import { I18nProvider } from "../i18n";
import { SecurityReviewPanel } from "../components/SecurityReviewPanel";
import { ChangeDetailPage } from "../components/ChangeDetailPage";
import type { SecurityInspection,SecurityReport,SecurityReviewAdapter } from "../security";
const report={formatVersion:1,reportId:"report-one",projectKey:"project-a",projectName:"Demo",reviewedAt:"2026-10-17",commitSha:null,ruleVersion:"1.0.0",scope:"web-api",executionMode:"builtin-offline",reviewMethods:["code"],complete:false,configurationState:"configured",counts:{checksSucceeded:1,checksFailed:1,findings:1,notVerified:1,notApplicable:0,disabled:0},findings:[{findingId:"one",ruleId:"SL-SEC-001",title:"Potential secret",severity:"medium",confidence:"low",path:"app.ts",line:3,evidence:"[REDACTED] <script>alert(1)</script>",description:"Needs human confirmation",remediation:"Use secure configuration",references:[],confirmationStatus:"needs_review",kind:"potential_risk"}],checks:[{ruleId:"owasp_top10",target:"",status:"NOT_VERIFIED",reason:"missing_data",relatedStandardIds:["owasp_top10"]}],standards:[{standardId:"owasp_top10",version:"2025",level:null,installedStatus:"missing",status:"NOT_VERIFIED"}],coverage:{inspectedFiles:1,skippedFiles:0,failedItems:0,bytesRead:42,remainingTargets:[],reasons:[]},disclaimer:"PASS only covers the executed rules",sourceLicenses:[],sources:[]} as SecurityReport;
const inspection={enabled:true,projectKey:"project-a",latest:report,requiresPreparation:false,selectedStandards:[{standardId:"owasp_top10",version:"2025"}]} as SecurityInspection;
function adapter():SecurityReviewAdapter{return {inspect:vi.fn().mockResolvedValue(inspection),start:vi.fn().mockResolvedValue({jobId:"job-one",projectKey:"project-a"}),wait:vi.fn().mockResolvedValue({reportId:"report-one",projectKey:"project-a"}),cancel:vi.fn().mockResolvedValue(undefined),report:vi.fn().mockResolvedValue(report),exportReport:vi.fn().mockResolvedValue(undefined),copySummary:vi.fn().mockResolvedValue(undefined),subscribe:vi.fn().mockResolvedValue(()=>{}),reveal:vi.fn().mockResolvedValue(undefined)};}
it("顯示標準、未驗證與遮罩證據，輸出及資料管理操作不開工單",async()=>{
 const api=adapter();const settings=vi.fn();render(<I18nProvider locale="zh-TW"><SecurityReviewPanel adapter={api} change="demo" inspection={inspection} onOpenDataSettings={settings}/></I18nProvider>);
 expect(screen.getByText(/不代表不存在其他漏洞/)).toBeTruthy();expect(screen.getAllByText(/2025/)[0]).toBeTruthy();expect(screen.getByText("[REDACTED] <script>alert(1)</script>")).toBeTruthy();expect(document.querySelector("script")).toBeNull();
 fireEvent.click(screen.getByRole("button",{name:"前往安全標準資料"}));expect(settings).toHaveBeenCalled();
 fireEvent.click(screen.getByRole("button",{name:"匯出 JSON"}));await waitFor(()=>expect(api.exportReport).toHaveBeenCalledWith("report-one","json"));
 fireEvent.click(screen.getByRole("button",{name:"匯出 Markdown"}));await waitFor(()=>expect(api.exportReport).toHaveBeenCalledWith("report-one","markdown"));
 fireEvent.click(screen.getByRole("button",{name:"複製遮罩摘要"}));await waitFor(()=>expect(api.copySummary).toHaveBeenCalledWith("report-one"));
});
it("取消與進度依 jobId/projectKey 隔離",async()=>{
 const api=adapter();let resolve!:(r:{reportId:string;projectKey:string})=>void;vi.mocked(api.wait).mockImplementation(()=>new Promise(r=>{resolve=r;}));
 render(<I18nProvider locale="zh-TW"><SecurityReviewPanel adapter={api} change="demo" inspection={{...inspection,latest:null}}/></I18nProvider>);
 fireEvent.click(screen.getByRole("button",{name:"執行基本審查"}));await waitFor(()=>expect(api.wait).toHaveBeenCalled());
 const listener=vi.mocked(api.subscribe).mock.calls[0][0];act(()=>listener({jobId:"another",projectKey:"project-b",phase:"wrong-event",completedBytes:99,completedFiles:1,totalBytes:null,message:"wrong-event"}));expect(screen.queryByText(/wrong-event/)).toBeNull();
 fireEvent.click(screen.getByRole("button",{name:"取消審查"}));await waitFor(()=>expect(api.cancel).toHaveBeenCalledWith({jobId:"job-one",projectKey:"project-a"}));
 await act(async()=>resolve({reportId:"report-one",projectKey:"project-a"}));expect(await screen.findByText("SL-SEC-001")).toBeTruthy();
});
it("沒有 Verify 工單也有獨立安全資訊；無 adapter 維持原行為",async()=>{
 const props={change:{name:"demo",status:"in-progress",totalTasks:1,completedTasks:1},loadDocument:vi.fn().mockResolvedValue(null),loadCapabilities:vi.fn().mockResolvedValue([]),loadMeta:vi.fn().mockResolvedValue(null)};
 const api=adapter();const {unmount}=render(<I18nProvider locale="zh-TW"><ChangeDetailPage {...props} securityReview={api}/></I18nProvider>);
 const tab=await screen.findByRole("tab",{name:/驗證/});fireEvent.mouseDown(tab,{button:0,ctrlKey:false});fireEvent.click(tab);expect(await screen.findByText("OWASP / CWE 初步安全審查")).toBeTruthy();expect(screen.queryByText("沒有驗證工單")).toBeNull();unmount();
 await act(async()=>{render(<I18nProvider locale="zh-TW"><ChangeDetailPage {...props}/></I18nProvider>);});expect(screen.queryByRole("tab",{name:/驗證/})).toBeNull();
});
