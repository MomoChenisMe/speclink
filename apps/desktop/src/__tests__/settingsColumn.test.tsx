// 設定模式的兩種分類欄（desktop-settings-mode design D2；spec「側欄導覽結構」）。
import { describe, it, expect, vi } from "vitest";
import { fireEvent, render as rtlRender, screen, within } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider } from "@speclink/ui";

import { SettingsColumn } from "../components/SettingsColumn";
import { ProjectSettingsColumn } from "../components/ProjectSettingsColumn";
import { APP_MESSAGES } from "../i18n/messages";

const zhWrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW" messages={APP_MESSAGES}>
    {children}
  </I18nProvider>
);
const render = (ui: ReactElement) => rtlRender(ui, { wrapper: zhWrapper });

const navItems = (nav: HTMLElement) =>
  within(nav)
    .getAllByRole("button")
    .map((b) => b.textContent);

describe("SettingsColumn", () => {
  it("「應用程式」群組四項＋「關於」群組一項依序渲染，所在分類為作用中，點擊回呼分類", () => {
    const onSelect = vi.fn();
    render(<SettingsColumn section="updates" onSelect={onSelect} />);
    const nav = screen.getByRole("navigation", { name: "設定分類" });
    expect(within(nav).getByText("應用程式")).toBeTruthy();
    expect(navItems(nav)).toEqual(["一般", "更新", "CLI", "伺服器", "關於 Speclink"]);
    expect(within(nav).getByRole("button", { name: "更新" }).getAttribute("aria-current")).toBe("page");
    fireEvent.click(within(nav).getByRole("button", { name: "伺服器" }));
    expect(onSelect).toHaveBeenCalledWith("servers");
  });

  it("「關於」群組在「應用程式」之後、唯一一項「關於 Speclink」；作用中與點擊回呼", () => {
    const onSelect = vi.fn();
    const { rerender } = render(<SettingsColumn section="general" onSelect={onSelect} />);
    const nav = screen.getByRole("navigation", { name: "設定分類" });
    const headers = [within(nav).getByText("應用程式"), within(nav).getByText("關於")];
    expect(headers[0].compareDocumentPosition(headers[1]) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    const about = within(nav).getByRole("button", { name: "關於 Speclink" });
    expect(headers[1].compareDocumentPosition(about) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    fireEvent.click(about);
    expect(onSelect).toHaveBeenCalledWith("about");
    rerender(<SettingsColumn section="about" onSelect={onSelect} />);
    expect(within(nav).getByRole("button", { name: "關於 Speclink" }).getAttribute("aria-current")).toBe("page");
  });

  it("搜尋只命中關於頁時只剩「關於」群組；群組無命中時連標題一起隱藏", () => {
    render(<SettingsColumn section="general" onSelect={() => {}} />);
    const nav = screen.getByRole("navigation", { name: "設定分類" });
    fireEvent.change(within(nav).getByPlaceholderText("搜尋設定"), { target: { value: "第三方" } });
    expect(navItems(nav)).toEqual(["關於 Speclink"]);
    expect(within(nav).queryByText("應用程式")).toBeNull();
    // 成為第一組時不帶群組間的上方間距。
    expect(within(nav).getByText("關於").closest(".pb-1")?.classList.contains("pt-3")).toBe(false);
  });

  it("檢查更新與更新日誌只命中更新頁（關於頁不設這兩列）", () => {
    render(<SettingsColumn section="general" onSelect={() => {}} />);
    const nav = screen.getByRole("navigation", { name: "設定分類" });
    const search = within(nav).getByPlaceholderText("搜尋設定");
    for (const query of ["檢查更新", "更新日誌"]) {
      fireEvent.change(search, { target: { value: query } });
      expect(navItems(nav), query).toEqual(["更新"]);
    }
  });

  it("搜尋「外觀」只剩一般且「關於」群組標題隱藏；無命中顯示提示；清空後五項還原", () => {
    render(<SettingsColumn section="general" onSelect={() => {}} />);
    const nav = screen.getByRole("navigation", { name: "設定分類" });
    const search = within(nav).getByPlaceholderText("搜尋設定");
    fireEvent.change(search, { target: { value: "外觀" } });
    expect(navItems(nav)).toEqual(["一般"]);
    expect(within(nav).queryByText("關於")).toBeNull();
    fireEvent.change(search, { target: { value: "zzz" } });
    expect(within(nav).queryAllByRole("button")).toHaveLength(0);
    expect(within(nav).getByText("沒有符合的設定")).toBeTruthy();
    fireEvent.change(search, { target: { value: "" } });
    expect(navItems(nav)).toHaveLength(5);
  });

  it("比對大小寫不敏感（分類名與列標籤）", () => {
    render(<SettingsColumn section="general" onSelect={() => {}} />);
    const nav = screen.getByRole("navigation", { name: "設定分類" });
    fireEvent.change(within(nav).getByPlaceholderText("搜尋設定"), { target: { value: "cli" } });
    // CLI 頁本身；「關於 Speclink」的分類名含 spe-cli-nk 也命中；一般頁已無 CLI 列。
    expect(navItems(nav)).toEqual(["CLI", "關於 Speclink"]);
  });
});

describe("ProjectSettingsColumn", () => {
  it("local：兩群組——規格資料夾的 config.yaml 四項與專案組 .speclink.yaml", () => {
    const onSelect = vi.fn();
    render(
      <ProjectSettingsColumn
        section="context"
        remote={false}
        configFile="speclink/config.yaml"
        configParseError={false}
        speclinkParseError={false}
        onSelect={onSelect}
      />,
    );
    const nav = screen.getByRole("navigation", { name: "專案設定分類" });
    expect(within(nav).getByText("speclink/config.yaml")).toBeTruthy();
    expect(within(nav).getByText("專案")).toBeTruthy();
    expect(navItems(nav)).toEqual(["專案說明", "產出規則", "審查規則", "產出流程", ".speclink.yaml"]);
    expect(within(nav).getByRole("button", { name: "專案說明" }).getAttribute("aria-current")).toBe("page");
    fireEvent.click(within(nav).getByRole("button", { name: ".speclink.yaml" }));
    expect(onSelect).toHaveBeenCalledWith("speclink");
  });

  it("舊佈局專案：群組名為 openspec/config.yaml", () => {
    render(
      <ProjectSettingsColumn section="context" remote={false} configFile="openspec/config.yaml" configParseError={false} speclinkParseError={false} onSelect={() => {}} />,
    );
    expect(within(screen.getByRole("navigation", { name: "專案設定分類" })).getByText("openspec/config.yaml")).toBeTruthy();
  });

  it("設定還沒載入：先不顯示規格資料夾組名，免得舊佈局專案閃一下預設名稱", () => {
    render(
      <ProjectSettingsColumn section="context" remote={false} configFile={null} configParseError={false} speclinkParseError={false} onSelect={() => {}} />,
    );
    const nav = screen.getByRole("navigation", { name: "專案設定分類" });
    expect(within(nav).queryByText(/config\.yaml/)).toBeNull();
    expect(navItems(nav)).toEqual(["專案說明", "產出規則", "審查規則", "產出流程", ".speclink.yaml"]);
  });

  it("remote：無專案組與 .speclink.yaml", () => {
    render(
      <ProjectSettingsColumn section="context" remote configFile="speclink/config.yaml" configParseError={false} speclinkParseError={false} onSelect={() => {}} />,
    );
    const nav = screen.getByRole("navigation", { name: "專案設定分類" });
    expect(navItems(nav)).toEqual(["專案說明", "產出規則", "審查規則", "產出流程"]);
    expect(within(nav).queryByText("專案")).toBeNull();
  });

  it("config.yaml 解析失敗時前四項帶警示點；.speclink.yaml 解析失敗時第五項帶警示點", () => {
    const { rerender } = render(
      <ProjectSettingsColumn section="context" remote={false} configFile="speclink/config.yaml" configParseError speclinkParseError={false} onSelect={() => {}} />,
    );
    const warned = () =>
      screen.getAllByTestId("tab-warning").map((dot) => dot.closest("button")?.getAttribute("aria-label") ?? dot.closest("button")?.textContent);
    expect(warned()).toEqual(["專案說明", "產出規則", "審查規則", "產出流程"]);
    rerender(
      <ProjectSettingsColumn section="context" remote={false} configFile="speclink/config.yaml" configParseError={false} speclinkParseError onSelect={() => {}} />,
    );
    expect(warned()).toEqual([".speclink.yaml"]);
  });

  it("底部 notice 槽渲染提示卡", () => {
    render(
      <ProjectSettingsColumn
        section="context"
        remote={false}
        configFile="speclink/config.yaml"
        configParseError={false}
        speclinkParseError={false}
        onSelect={() => {}}
        notice={<div>技能檔是舊版</div>}
      />,
    );
    expect(screen.getByText("技能檔是舊版")).toBeTruthy();
  });
});
