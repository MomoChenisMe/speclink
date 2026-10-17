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

vi.mock("../adapter/security", () => ({listSecurityData:vi.fn(),startSecurityData:vi.fn(),waitSecurityData:vi.fn(),cancelSecurityJob:vi.fn(),watchSecurityProgress:vi.fn(),chooseSecurityPackage:vi.fn()}));
const inventory = {standards:[{standardId:"owasp_top10",versions:[{version:"2025",status:"missing",sourceUrl:"https://github.com/OWASP/Top10",licenseName:"CC-BY-SA-4.0",licenseUrl:"https://creativecommons.org/licenses/by-sa/4.0/",attribution:"OWASP Foundation",sourceFormat:"markdown",error:null}]}]};
const mount = () => render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><SecurityDataPage /></I18nProvider>);
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(openUrl).mockResolvedValue(undefined);
  vi.mocked(api.listSecurityData).mockResolvedValue(structuredClone(inventory));
  vi.mocked(api.watchSecurityProgress).mockResolvedValue(() => {});
  vi.mocked(api.startSecurityData).mockResolvedValue({jobId:"job-one",projectKey:"application-data"});
  vi.mocked(api.waitSecurityData).mockResolvedValue({});
  vi.mocked(api.cancelSecurityJob).mockResolvedValue(true);
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
