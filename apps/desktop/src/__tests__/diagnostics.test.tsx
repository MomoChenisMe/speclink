import {beforeEach,expect,it,vi} from "vitest";
import {act,fireEvent,render,screen,waitFor} from "@testing-library/react";
import {I18nProvider} from "@speclink/ui";
import {DiagnosticsBoundary} from "../diagnostics";
import {APP_MESSAGES} from "../i18n/messages";
import * as api from "../adapter/runtime";
vi.mock("../adapter/runtime",()=>({readRuntimePreferences:vi.fn(),appendDiagnostic:vi.fn()}));
beforeEach(()=>{vi.resetAllMocks();vi.mocked(api.appendDiagnostic).mockResolvedValue(undefined);});
const snapshot=(enabled:boolean)=>({formatVersion:1 as const,debugEnabled:enabled,proxy:{mode:"system" as const,url:null},readError:null,diagnosticsStatus:enabled?"recording" as const:"off" as const});
it.each([true, false])("偏好載入前的錯誤與 rejection 依最後 Debug=%s 決定送出，暫存最多16筆", async (enabled) => {
 let resolve!: (value: ReturnType<typeof snapshot>) => void;
 vi.mocked(api.readRuntimePreferences).mockReturnValue(new Promise(done => { resolve = done; }));
 render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><DiagnosticsBoundary><span>內容</span></DiagnosticsBoundary></I18nProvider>);
 for (let index = 0; index < 20; index++) {
  if (index % 2 === 0) fireEvent(window, new ErrorEvent("error", { error: new TypeError("SECRET") }));
  else {
   const event = new Event("unhandledrejection");
   Object.defineProperty(event, "reason", { value: new Error("SECRET") });
   fireEvent(window, event);
  }
 }
 expect(api.appendDiagnostic).not.toHaveBeenCalled();
 await act(async () => { resolve(snapshot(enabled)); });
 expect(api.appendDiagnostic).toHaveBeenCalledTimes(enabled ? 16 : 0);
 if (enabled) {
  expect(vi.mocked(api.appendDiagnostic).mock.calls.map(([entry]) => entry.event)).toEqual(Array.from({length:16}, (_, i) => i % 2 === 0 ? "unhandled-error" : "unhandled-rejection"));
  expect(JSON.stringify(vi.mocked(api.appendDiagnostic).mock.calls)).not.toContain("SECRET");
 }
});
it.each([true,false])("未處理錯誤只在Debug=%s時記錄且不傳秘密message",async(enabled)=>{
 vi.mocked(api.readRuntimePreferences).mockResolvedValue(snapshot(enabled));
 render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><DiagnosticsBoundary><span>內容</span></DiagnosticsBoundary></I18nProvider>);
 await waitFor(()=>expect(api.readRuntimePreferences).toHaveBeenCalled());
 await act(async()=>{});
 fireEvent(window,new ErrorEvent("error",{error:new TypeError("Bearer SECRET /Users/SECRET/private"),message:"SECRET"}));
 if(enabled) {
  await waitFor(()=>expect(api.appendDiagnostic).toHaveBeenCalled());
  expect(JSON.stringify(vi.mocked(api.appendDiagnostic).mock.calls)).not.toContain("SECRET");
 } else expect(api.appendDiagnostic).not.toHaveBeenCalled();
});
it("畫面錯誤有重試及診斷出口，開機已啟用也能記錄首次render失敗",async()=>{
 vi.mocked(api.readRuntimePreferences).mockResolvedValue(snapshot(true));
 const spy=vi.spyOn(console,"error").mockImplementation(()=>{});
 let broken=true;
 const Faulty=()=>{if(broken)throw new TypeError("SECRET");return <span>恢復</span>;};
 try {
  render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><DiagnosticsBoundary><Faulty/></DiagnosticsBoundary></I18nProvider>);
  expect(await screen.findByRole("button",{name:"前往網路與診斷"})).toBeTruthy();
  await waitFor(()=>expect(api.appendDiagnostic).toHaveBeenCalled());
  fireEvent.click(screen.getByRole("button",{name:"前往網路與診斷"}));
  expect(await screen.findByRole("heading",{name:"網路與診斷"})).toBeTruthy();
  broken=false;fireEvent.click(screen.getByRole("button",{name:"重新載入畫面"}));
  expect(await screen.findByText("恢復")).toBeTruthy();
  expect(JSON.stringify(vi.mocked(api.appendDiagnostic).mock.calls)).not.toContain("SECRET");
 }finally{spy.mockRestore();}
});

it("只記錄目前程式腳本的相對位置，停錄及卸載後不送事件",async()=>{
 vi.mocked(api.readRuntimePreferences).mockResolvedValue(snapshot(true));
 const script=document.createElement("script");script.src=`${window.location.origin}/assets/index-12345678.js`;document.head.append(script);
 const view=render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><DiagnosticsBoundary><span>內容</span></DiagnosticsBoundary></I18nProvider>);
 await act(async()=>{});
 fireEvent(window,new ErrorEvent("error",{error:new TypeError("SECRET"),filename:script.src,lineno:12,colno:3}));
 await waitFor(()=>expect(api.appendDiagnostic).toHaveBeenCalled());
 expect(vi.mocked(api.appendDiagnostic).mock.calls[0][0].errorChain).toContainEqual({kind:"frontend",code:"app:assets/index-12345678.js:12:3"});
 act(()=>window.dispatchEvent(new CustomEvent("speclink:runtime-preferences",{detail:snapshot(false)})));
 const before=vi.mocked(api.appendDiagnostic).mock.calls.length;
 fireEvent(window,new ErrorEvent("error",{error:new Error("SECRET")}));expect(api.appendDiagnostic).toHaveBeenCalledTimes(before);
 const removed=vi.spyOn(window,"removeEventListener");view.unmount();expect(removed).toHaveBeenCalledWith("error",expect.any(Function));expect(removed).toHaveBeenCalledWith("unhandledrejection",expect.any(Function));expect(api.appendDiagnostic).toHaveBeenCalledTimes(before);removed.mockRestore();script.remove();
});
