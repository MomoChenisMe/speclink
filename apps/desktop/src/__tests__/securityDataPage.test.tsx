import { beforeEach, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { I18nProvider } from "@speclink/ui";
import { SecurityDataPage } from "../views/settings/SecurityDataPage";
import { APP_MESSAGES } from "../i18n/messages";
import * as api from "../adapter/security";
import { openUrl } from "@tauri-apps/plugin-opener";
import { toast } from "sonner";
import { readFileSync } from "node:fs";

vi.mock("@tauri-apps/plugin-opener", () => ({openUrl:vi.fn()}));
vi.mock("sonner", () => ({toast:{error:vi.fn()}}));

vi.mock("../adapter/security", () => ({readSecurityDataPaths:vi.fn(),writeSecurityDataPaths:vi.fn(),chooseSecurityDataDirectory:vi.fn(),listSecurityData:vi.fn(),startSecurityData:vi.fn(),waitSecurityData:vi.fn(),cancelSecurityJob:vi.fn(),watchSecurityProgress:vi.fn(),chooseSecurityPackage:vi.fn(),chooseSecurityExport:vi.fn()}));
const inventory = {standards:[{standardId:"owasp_top10",versions:[{version:"2025",status:"missing",sourceUrl:"https://github.com/OWASP/Top10",licenseName:"CC-BY-SA-4.0",licenseUrl:"https://creativecommons.org/licenses/by-sa/4.0/",attribution:"OWASP Foundation",sourceFormat:"markdown",error:null}]}]};
const mount = () => render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><SecurityDataPage /></I18nProvider>);
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.readSecurityDataPaths).mockResolvedValue({standardsPath:null,effectivePath:"/device/security/standards",isDefault:true,access:"readWrite"});
  vi.mocked(openUrl).mockResolvedValue(undefined);
  vi.mocked(api.listSecurityData).mockResolvedValue(structuredClone(inventory));
  vi.mocked(api.watchSecurityProgress).mockResolvedValue(() => {});
  vi.mocked(api.startSecurityData).mockResolvedValue({jobId:"job-one",projectKey:"application-data"});
  vi.mocked(api.waitSecurityData).mockResolvedValue({});
  vi.mocked(api.cancelSecurityJob).mockResolvedValue(true);
});

it("UNC路徑儲存失敗保留草稿，唯讀共用只准讀取／匯出，還原不搬檔",async()=>{
  const share=String.raw`\\fileserver\team\standards`;
  mount();const input=await screen.findByRole("textbox",{name:"標準資料存放路徑"});
  fireEvent.change(input,{target:{value:share}});
  vi.mocked(api.writeSecurityDataPaths).mockRejectedValueOnce({message:"settings_write_failed"});
  fireEvent.click(screen.getByRole("button",{name:"儲存路徑"}));
  expect((await screen.findByRole("alert")).textContent).toContain("settings_write_failed");
  expect((input as HTMLInputElement).value).toBe(share);
  vi.mocked(api.writeSecurityDataPaths).mockResolvedValueOnce({standardsPath:share,effectivePath:share,isDefault:false,access:"readOnly"});
  fireEvent.click(screen.getByRole("button",{name:"儲存路徑"}));
  expect(await screen.findByText("唯讀：可以查詢及匯出，不能修改資料。" )).toBeTruthy();
  expect((screen.getByRole("button",{name:"匯入離線資料包"}) as HTMLButtonElement).disabled).toBe(true);
  expect((screen.getByRole("button",{name:"全部匯出"}) as HTMLButtonElement).disabled).toBe(false);
  vi.mocked(api.writeSecurityDataPaths).mockResolvedValueOnce({standardsPath:null,effectivePath:"/device/security/standards",isDefault:true,access:"readWrite"});
  fireEvent.click(screen.getByRole("button",{name:"還原本機預設"}));
  await waitFor(()=>expect(api.writeSecurityDataPaths).toHaveBeenLastCalledWith(null));
  expect(api.startSecurityData).not.toHaveBeenCalled();
});

it.each(["zh-TW","en"] as const)("%s：選目錄與取消只改草稿，離線路徑停用資料操作",async(locale)=>{
  const m=APP_MESSAGES[locale];
  vi.mocked(api.readSecurityDataPaths).mockResolvedValue({standardsPath:"/offline/share",effectivePath:"/offline/share",isDefault:false,access:"unavailable"});
  vi.mocked(api.chooseSecurityDataDirectory).mockResolvedValue("/chosen/share");
  render(<I18nProvider locale={locale} messages={APP_MESSAGES}><SecurityDataPage/></I18nProvider>);
  const input=await screen.findByRole("textbox",{name:m["securityData.path.title"]});
  await waitFor(()=>expect((input as HTMLInputElement).value).toBe("/offline/share"));
  expect((screen.getByRole("button",{name:m["securityData.exportAll"]}) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByRole("button",{name:m["securityData.path.choose"]}));
  await waitFor(()=>expect((input as HTMLInputElement).value).toBe("/chosen/share"));
  fireEvent.click(screen.getByRole("button",{name:m["securityData.path.cancel"]}));
  expect((input as HTMLInputElement).value).toBe("/offline/share");
  expect(api.writeSecurityDataPaths).not.toHaveBeenCalled();expect(api.startSecurityData).not.toHaveBeenCalled();
});

it("儲存新資料路徑後忽略舊路徑延遲回來的清單",async()=>{
  vi.mocked(api.writeSecurityDataPaths).mockResolvedValue({standardsPath:"/new/standards",effectivePath:"/new/standards",isDefault:false,access:"readWrite"});
  let finishOld!:(value:typeof inventory)=>void;
  vi.mocked(api.listSecurityData)
    .mockImplementationOnce(()=>new Promise(resolve=>{finishOld=resolve;}))
    .mockResolvedValueOnce({standards:[{...inventory.standards[0],versions:[{...inventory.standards[0].versions[0],status:"installed"}]}]});
  mount();
  const input=await screen.findByRole("textbox",{name:"標準資料存放路徑"});
  fireEvent.change(input,{target:{value:"/new/standards"}});
  fireEvent.click(screen.getByRole("button",{name:"儲存路徑"}));
  await waitFor(()=>expect(api.listSecurityData).toHaveBeenCalledTimes(2));
  expect(await screen.findByText("已安裝")).toBeTruthy();
  await act(async()=>finishOld(structuredClone(inventory)));
  expect(screen.getByText("已安裝")).toBeTruthy();
});

it("零專案進頁只讀清單，來源授權可讀，下載必須明確操作", async () => {
  mount();
  expect(await screen.findByRole("heading",{name:"安全標準資料"})).toBeTruthy();
  expect(await screen.findByText("OWASP Foundation")).toBeTruthy();
  expect(screen.getByRole("link",{name:"CC-BY-SA-4.0"}).getAttribute("href")).toContain("creativecommons.org");
  expect(api.startSecurityData).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button",{name:"下載"}));
  await waitFor(() => expect(api.startSecurityData).toHaveBeenCalledWith({action:"install",standardId:"owasp_top10",version:"2025",reinstall:false}));
});

it("重新下載失敗與使用中刪除保留已安裝狀態及原因", async () => {
  vi.mocked(api.listSecurityData).mockResolvedValue({standards:[{...inventory.standards[0],versions:[{...inventory.standards[0].versions[0],status:"installed"}]}]});
  vi.mocked(api.waitSecurityData).mockRejectedValue({code:"in_use",message:"in_use: selected by a locally known policy"});
  mount();
  fireEvent.click(await screen.findByRole("button",{name:"刪除未使用版本"}));
  expect((await screen.findByRole("alert")).textContent).toContain("in_use");
  expect(screen.getByText("已安裝")).toBeTruthy();
  expect(screen.getByRole("button",{name:"重新下載"})).toBeTruthy();
});

it("進度只採目前工作，取消等待完成，不把取消顯示為已安裝", async () => {
  let finish!: (value: unknown) => void;
  vi.mocked(api.waitSecurityData).mockImplementation(() => new Promise((resolve) => {finish = resolve;}));
  mount();
  fireEvent.click(await screen.findByRole("button",{name:"下載"}));
  await waitFor(() => expect(api.waitSecurityData).toHaveBeenCalledWith("job-one"));
  const callback = vi.mocked(api.watchSecurityProgress).mock.calls[0][0];
  act(()=>callback({jobId:"other-job",projectKey:"other-project",phase:"downloading",completedBytes:999,totalBytes:null,completedFiles:0,message:"wrong-event"}));
  expect(screen.queryByText(/wrong-event/)).toBeNull();
  fireEvent.click(screen.getByRole("button",{name:"取消"}));
  await waitFor(() => expect(api.cancelSecurityJob).toHaveBeenCalledWith("job-one"));
  await act(async()=>finish({cancelled:true}));
  await waitFor(() => expect(screen.queryByRole("button",{name:"取消"})).toBeNull());
  expect(screen.queryByText("已安裝")).toBeNull();
});

it.each(["missing","installed"])("%s：下載時僅當次 API 版本顯示下載中，失敗後還原",async(status)=>{
  vi.mocked(api.listSecurityData).mockResolvedValue({standards:[{standardId:"owasp_api_top10",versions:["2019","2023"].map(version=>({...inventory.standards[0].versions[0],version,status}))}]});
  let fail!:(reason:unknown)=>void;
  vi.mocked(api.startSecurityData).mockImplementation(()=>new Promise(()=>{}));
  mount();const buttons=await screen.findAllByRole("button",{name:status==="missing"?"下載":"重新下載"});
  // Feedback must appear before the start IPC returns a job ticket.
  const started=Promise.resolve({jobId:"job-api",projectKey:"application-data"});
  vi.mocked(api.startSecurityData).mockReturnValue(started);
  vi.mocked(api.waitSecurityData).mockImplementation(()=>new Promise((_,reject)=>{fail=reject;}));
  fireEvent.click(buttons[0]);expect(buttons[0].textContent).toBe("下載中");
  expect(buttons[1].textContent).toBe(status==="missing"?"下載":"重新下載");expect(buttons[1].hasAttribute("disabled")).toBe(true);
  await waitFor(()=>expect(api.waitSecurityData).toHaveBeenCalled());
  await act(async()=>fail({message:"download_failed"}));
  await waitFor(()=>expect(buttons[0].textContent).toBe(status==="missing"?"下載":"重新下載"));
});

it("官方來源及授權連結沿用系統瀏覽器與失敗提示",async()=>{
 mount();
 const source=await screen.findByRole("link",{name:inventory.standards[0].versions[0].sourceUrl});
 const license=screen.getByRole("link",{name:"CC-BY-SA-4.0"});
 fireEvent.click(source);
 await waitFor(()=>expect(openUrl).toHaveBeenCalledWith(source.getAttribute("href")));
 vi.mocked(openUrl).mockRejectedValueOnce("opener denied");
 fireEvent.click(license);
 await waitFor(()=>expect(toast.error).toHaveBeenCalledWith("opener denied"));
});

it("桌面外開權限包含標準清單中的官方來源與授權",()=>{
 const capability=JSON.parse(readFileSync("src-tauri/capabilities/default.json","utf8"));
 const allowed=capability.permissions.find((p:unknown)=>typeof p==="object"&&p!==null&&"identifier" in p&&p.identifier==="opener:allow-open-url").allow as {url:string}[];
 const sources=JSON.parse(readFileSync("../../crates/engine/speclink-core/assets/security/sources.json","utf8")) as {sourceUrl:string;licenseUrl:string}[];
 for(const source of sources)for(const url of [source.sourceUrl,source.licenseUrl]) {
  expect(allowed.some((pattern)=>pattern.url.endsWith("*")?url.startsWith(pattern.url.slice(0,-1)):url===pattern.url)).toBe(true);
 }
 expect(allowed.some((pattern)=>pattern.url==="https://*"||pattern.url==="*")).toBe(false);
});

it("單一及全部匯出先選路徑，取消不啟動工作，成功顯示位置",async()=>{
 vi.mocked(api.listSecurityData).mockResolvedValue({standards:[{...inventory.standards[0],versions:[{...inventory.standards[0].versions[0],status:"installed"}]}]});
 vi.mocked(api.chooseSecurityExport).mockResolvedValue(null);
 mount();fireEvent.click(await screen.findByRole("button",{name:"全部匯出"}));
 await waitFor(()=>expect(api.chooseSecurityExport).toHaveBeenCalled());expect(api.startSecurityData).not.toHaveBeenCalled();
 vi.mocked(api.chooseSecurityExport).mockResolvedValue("/exports/one.zip");
 vi.mocked(api.waitSecurityData).mockResolvedValue({path:"/exports/one.zip",formatVersion:1,bytes:42,packages:[],skipped:[]});
 fireEvent.click(screen.getByRole("button",{name:"匯出此版本"}));
 await waitFor(()=>expect(api.startSecurityData).toHaveBeenCalledWith({action:"export",standardId:"owasp_top10",version:"2025",path:"/exports/one.zip"}));
 expect(await screen.findByText(/已匯出至.*one.zip/)).toBeTruthy();
 vi.mocked(api.chooseSecurityExport).mockResolvedValue("/exports/all.zip");
 fireEvent.click(screen.getByRole("button",{name:"全部匯出"}));
 await waitFor(()=>expect(api.startSecurityData).toHaveBeenCalledWith({action:"export",all:true,path:"/exports/all.zip"}));
});
it("集合匯入顯示部分成功與每項失敗，刷新清單",async()=>{
 vi.mocked(api.chooseSecurityPackage).mockResolvedValue("/offline/batch.zip");
 vi.mocked(api.waitSecurityData).mockResolvedValue({formatVersion:2,status:"partial",results:[{standardId:"cwe",version:"4.20",status:"installed",sha256:"abc",errorCode:null},{standardId:"owasp_top10",version:"2025",status:"failed",sha256:null,errorCode:"busy"},{standardId:"owasp_asvs",version:"5.0.0",status:"notAttempted",sha256:null,errorCode:null}]});
 mount();fireEvent.click(await screen.findByRole("button",{name:"匯入離線資料包"}));
 expect(await screen.findByText("部分完成")).toBeTruthy();expect(screen.getByText(/2025.*失敗.*busy/)).toBeTruthy();expect(screen.getByText(/5.0.0.*尚未處理/)).toBeTruthy();
 expect(api.listSecurityData).toHaveBeenCalledTimes(2);
});
