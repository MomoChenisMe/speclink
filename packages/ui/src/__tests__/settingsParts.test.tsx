// 五個設定零件（desktop-settings-mode design D3）：頁標題、卡、列、列右側下拉、群組標題。
import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Globe } from "lucide-react";

import {
  SettingsCard,
  SettingsChoice,
  SettingsGroupHeader,
  SettingsLinkRow,
  SettingsPageHeader,
  SettingsRow,
} from "../components/ui/settings";

const classes = (el: Element) => (el.getAttribute("class") ?? "").split(/\s+/);

describe("SettingsPageHeader", () => {
  it("渲染標題、說明、右側動作與其下 12px 灰字註記", () => {
    render(
      <SettingsPageHeader
        title="一般"
        description="介面語言、更新與 CLI。"
        actions={<button>檢查更新</button>}
        note="此頁內容僅存於此裝置，不寫入版本庫"
      />,
    );
    expect(screen.getByRole("heading", { level: 2, name: "一般" })).toBeTruthy();
    expect(screen.getByText("介面語言、更新與 CLI。")).toBeTruthy();
    expect(screen.getByRole("button", { name: "檢查更新" })).toBeTruthy();
    const note = screen.getByText("此頁內容僅存於此裝置，不寫入版本庫");
    expect(classes(note)).toEqual(expect.arrayContaining(["text-xs", "text-muted-foreground"]));
  });

  it("未給 note 時不渲染註記列", () => {
    const { container } = render(<SettingsPageHeader title="更新" />);
    expect(container.querySelector("[data-settings-note]")).toBeNull();
  });
});

describe("SettingsGroupHeader", () => {
  it("12px 粗體灰字標籤，可帶圖示與計數", () => {
    const { container } = render(
      <SettingsGroupHeader icon={<Globe data-testid="icon" />} label="應用程式" count={5} />,
    );
    const label = screen.getByText("應用程式");
    const root = container.firstElementChild as HTMLElement;
    expect(classes(root)).toEqual(
      expect.arrayContaining(["text-xs", "font-semibold", "text-muted-foreground"]),
    );
    expect(root.contains(label)).toBe(true);
    expect(screen.getByTestId("icon")).toBeTruthy();
    expect(screen.getByText("5")).toBeTruthy();
  });

  it("圖示與計數皆選配", () => {
    const { container } = render(<SettingsGroupHeader label="專案" />);
    expect(container.querySelector("svg")).toBeNull();
    expect(container.textContent).toBe("專案");
  });
});

describe("SettingsCard", () => {
  it("14px 小標（含計數）在 16px 圓角卡之上，卡以小標為區域名，列間細線", () => {
    render(
      <SettingsCard title="連線" count={2}>
        <SettingsRow label="甲" control={<span />} />
        <SettingsRow label="乙" control={<span />} />
      </SettingsCard>,
    );
    const region = screen.getByRole("region", { name: "連線" });
    const title = within(region).getByText("連線");
    expect(classes(title.parentElement as HTMLElement)).toContain("text-sm");
    expect(within(region).getByText("2")).toBeTruthy();
    const card = region.querySelector("[data-settings-card-body]") as HTMLElement;
    expect(classes(card)).toEqual(expect.arrayContaining(["rounded-2xl", "divide-y"]));
    expect(within(card).getAllByRole("group")).toHaveLength(2);
  });
});

describe("SettingsRow", () => {
  it("左標籤＋說明、右控制項；以標籤為群組名", () => {
    render(
      <SettingsRow
        label="外觀"
        description="跟隨系統或固定淺色／深色。"
        control={<button>深色</button>}
      />,
    );
    const row = screen.getByRole("group", { name: "外觀" });
    expect(classes(row)).toEqual(expect.arrayContaining(["grid", "grid-cols-[1fr_auto]", "px-4", "py-3"]));
    expect(classes(within(row).getByText("外觀"))).toContain("text-sm");
    const description = within(row).getByText("跟隨系統或固定淺色／深色。");
    expect(classes(description)).toEqual(expect.arrayContaining(["text-xs", "text-muted-foreground"]));
    expect(within(row).getByRole("button", { name: "深色" })).toBeTruthy();
  });

  it("warning 時說明改警示色", () => {
    render(<SettingsRow label="speclink CLI" description="版本不符" control={null} warning />);
    const description = screen.getByText("版本不符");
    expect(classes(description)).toContain("text-status-warning");
    expect(classes(description)).not.toContain("text-muted-foreground");
  });

  it("htmlFor 讓標籤對到控制項；children 渲染在列下方", () => {
    render(
      <SettingsRow label="locale" htmlFor="cfg-x" control={<input id="cfg-x" />}>
        <p>展開內容</p>
      </SettingsRow>,
    );
    expect(screen.getByLabelText("locale").id).toBe("cfg-x");
    expect(screen.getByText("展開內容")).toBeTruthy();
  });
});

describe("SettingsLinkRow", () => {
  it("整列為按鈕（hover 灰底、右端外開圖示），點擊回呼", async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    render(<SettingsLinkRow label="回報問題" description="開新 issue 並帶入版本資訊" onClick={onClick} />);
    const row = screen.getByRole("button", { name: /回報問題/ });
    expect(classes(row)).toEqual(expect.arrayContaining(["w-full", "text-left", "hover:bg-muted"]));
    expect(row.querySelector("svg.lucide-external-link")).toBeTruthy();
    expect(within(row).getByText("開新 issue 並帶入版本資訊")).toBeTruthy();
    await user.click(row);
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});

describe("SettingsChoice", () => {
  const OPTIONS = [
    { value: "system", label: "跟隨系統" },
    { value: "light", label: "淺色" },
    { value: "dark", label: "深色" },
  ];

  it("以 aria-label 可選、顯示現值、選取後回呼", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<SettingsChoice value="system" options={OPTIONS} onChange={onChange} ariaLabel="外觀" />);
    const trigger = screen.getByRole("combobox", { name: "外觀" });
    expect(trigger.textContent).toContain("跟隨系統");
    expect(classes(trigger)).toEqual(expect.arrayContaining(["h-8", "text-[13px]"]));
    await user.click(trigger);
    await user.click(await screen.findByRole("option", { name: "深色" }));
    expect(onChange).toHaveBeenCalledWith("dark");
  });

  it("停用項不可選、停用時觸發鈕 disabled", () => {
    render(
      <SettingsChoice
        value="繁體中文"
        options={[{ value: "繁體中文", label: "繁體中文（無效）", disabled: true }, ...OPTIONS]}
        onChange={() => {}}
        ariaLabel="locale"
        disabled
      />,
    );
    const trigger = screen.getByRole("combobox", { name: "locale" }) as HTMLButtonElement;
    expect(trigger.disabled).toBe(true);
    expect(trigger.textContent).toContain("繁體中文（無效）");
  });
});
