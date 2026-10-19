import {beforeEach,expect,it,vi} from "vitest";
import {fireEvent,render,screen,waitFor} from "@testing-library/react";
import {I18nProvider} from "@speclink/ui";
import {APP_MESSAGES} from "../i18n/messages";
import {NetworkDiagnosticsPage} from "../views/settings/NetworkDiagnosticsPage";
import * as api from "../adapter/runtime";
vi.mock("../adapter/runtime",()=>({readRuntimePreferences:vi.fn(),writeRuntimePreferences:vi.fn(),exportDiagnostics:vi.fn(),probeUpdate:vi.fn()}));
const initial:api.RuntimeSnapshot={formatVersion:1,debugEnabled:false,proxy:{mode:"system",url:null},readError:null,diagnosticsStatus:"off"};
const mount=()=>render(<I18nProvider locale="zh-TW" messages={APP_MESSAGES}><NetworkDiagnosticsPage/></I18nProvider>);
beforeEach(()=>{vi.resetAllMocks();vi.mocked(api.readRuntimePreferences).mockResolvedValue(initial);vi.mocked(api.writeRuntimePreferences).mockImplementation(async p=>({...p,readError:null,diagnosticsStatus:p.debugEnabled?"recording":"off"}));});
it("零專案啟用診斷，儲存後才顯示測試按鈕並可匯出",async()=>{
 mount();fireEvent.click(await screen.findByRole("switch",{name:"Debug mode"}));
 fireEvent.click(screen.getByRole("button",{name:"儲存"}));
 await waitFor(()=>expect(api.writeRuntimePreferences).toHaveBeenCalledWith({formatVersion:1,proxy:initial.proxy,debugEnabled:true}));
 expect(await screen.findByRole("button",{name:"測試更新連線"})).toBeTruthy();
 vi.mocked(api.exportDiagnostics).mockResolvedValue(null);
 fireEvent.click(screen.getByRole("button",{name:"匯出診斷紀錄"}));await waitFor(()=>expect(api.exportDiagnostics).toHaveBeenCalled());
});
it("偏好損壞顯示原因與明確重設，寫入失敗不冒稱儲存",async()=>{
 vi.mocked(api.readRuntimePreferences).mockResolvedValue({...initial,readError:"preferences_invalid"});
 mount();expect(await screen.findByRole("button",{name:"重設網路與診斷設定"})).toBeTruthy();
 vi.mocked(api.writeRuntimePreferences).mockRejectedValue({code:"settings_write_failed",message:"settings_write_failed"});
 fireEvent.click(screen.getByRole("button",{name:"重設網路與診斷設定"}));
 expect((await screen.findByRole("alert")).textContent).toContain("settings_write_failed");
});
