// design D1／D3／D7：詳情頁三欄骨架與標題列動作插槽。
import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { useRef, useState, type ReactNode } from "react";

import { I18nProvider, useI18n } from "../i18n";
import {
  DetailActions,
  DetailActionsSlot,
  DetailActionsSlotProvider,
  DetailIdentity,
  DetailShell,
  DocBody,
  DocTiles,
} from "../components/DetailShell";
import { TooltipProvider } from "../components/ui/tooltip";

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nProvider locale="zh-TW">
    <TooltipProvider>{children}</TooltipProvider>
  </I18nProvider>
);

function Shell() {
  const bodyRef = useRef<HTMLDivElement>(null);
  return (
    <DetailShell tree={<div>樹</div>} activeDoc="proposal" tocKey="k" bodyRef={bodyRef}>
      <DocBody ref={bodyRef}>
        <h2 data-doc-heading>為什麼</h2>
      </DocBody>
    </DetailShell>
  );
}

describe("DetailShell 三欄外框", () => {
  it("左欄文件樹、中欄閱讀區、右欄本頁目錄；右欄寬度門檻以 container query 控制", () => {
    const { container } = render(<Shell />, { wrapper });
    const shell = container.querySelector("[data-detail-shell]") as HTMLElement;
    expect(shell.className).toContain("@container");
    expect(shell.querySelector("[data-detail-tree]")!.textContent).toBe("樹");
    const reading = shell.querySelector("[data-detail-reading]") as HTMLElement;
    expect(reading.querySelector("[data-doc-body]")).toBeTruthy();
    const toc = shell.querySelector("[data-detail-toc]") as HTMLElement;
    expect(toc.className).toContain("hidden");
    expect(toc.className).toContain("@min-[1200px]:flex");
    // 本頁目錄掃 data-doc-body 內的錨點。
    expect(within(toc).getByRole("button", { name: "為什麼" })).toBeTruthy();
  });
});

describe("DetailShell 本頁目錄隨語系重掃", () => {
  it("詳情頁開著時切換語系，目錄錨點文字換成新語系", () => {
    function TranslatedShell() {
      const { t } = useI18n();
      const bodyRef = useRef<HTMLDivElement>(null);
      return (
        <DetailShell tree={null} activeDoc="proposal" tocKey="k" bodyRef={bodyRef}>
          <DocBody ref={bodyRef}>
            <h2 data-doc-heading>{t("detail.tree")}</h2>
          </DocBody>
        </DetailShell>
      );
    }
    const at = (locale: "zh-TW" | "en") => (
      <I18nProvider locale={locale}>
        <TooltipProvider>
          <TranslatedShell />
        </TooltipProvider>
      </I18nProvider>
    );
    const { container, rerender } = render(at("zh-TW"));
    const toc = () => container.querySelector("[data-detail-toc]") as HTMLElement;
    expect(within(toc()).getByRole("button", { name: "文件" })).toBeTruthy();
    rerender(at("en"));
    expect(within(toc()).getByRole("button", { name: "Documents" })).toBeTruthy();
  });
});

describe("DocTiles 數字磚", () => {
  it("空陣列不渲染", () => {
    const { container } = render(<DocTiles tiles={[]} />, { wrapper });
    expect(container.innerHTML).toBe("");
  });

  it("四格時渲染四個值與標籤", () => {
    const tiles = [
      { value: 3, label: "需求" },
      { value: 7, label: "情境" },
      { value: "9/9", label: "任務" },
      { value: 2, label: "決策" },
    ];
    const { container } = render(<DocTiles tiles={tiles} />, { wrapper });
    const cells = container.querySelectorAll("[data-doc-tile]");
    expect(cells).toHaveLength(4);
    expect(Array.from(cells).map((c) => c.firstElementChild!.textContent)).toEqual(["3", "7", "9/9", "2"]);
  });

  // spec「markdown 文件內容行寬有上限」的「數字磚依格數平分整列」：每格一欄、各欄等寬、填滿整列（1 格也佔滿）。
  // jsdom 不算版面，以 class 斷言欄的分配方式。
  it.each([1, 3, 4])("%i 格時數字磚列每格一欄、各欄等寬", (count) => {
    const tiles = Array.from({ length: count }, (_, i) => ({ value: i, label: `格 ${i}` }));
    const { container } = render(<DocTiles tiles={tiles} />, { wrapper });
    const cls = (container.querySelector("[data-doc-tiles]") as HTMLElement).className.split(" ");
    expect(cls).toContain("grid-flow-col");
    expect(cls).toContain("auto-cols-fr");
    expect(cls).not.toContain("grid-cols-4");
    expect(container.querySelectorAll("[data-doc-tile]")).toHaveLength(count);
  });
});

// spec「markdown 內容保留文件結構呈現」「markdown 文件內容行寬有上限」（design D10）：閱讀卡內文 15px／1.75，
// 行寬以卡欄為上限（不再套 96ch）。index.css 的 .markdown 基準字級不在 cascade layer、會壓過 Tailwind
// utilities 層的規則，所以 .markdown 的字級與行高覆蓋要帶 important（`!`）；jsdom 不算樣式，只斷言 class token。
describe("DocBody 字級與行寬", () => {
  it("內文 15px、行高 1.75，以 important 覆蓋 .markdown 字級，不帶 96ch 上限", () => {
    const { container } = render(<DocBody>內文</DocBody>, { wrapper });
    const cls = (container.querySelector("[data-doc-body]") as HTMLElement).className.split(" ");
    for (const token of [
      "text-[15px]",
      "leading-[1.75]",
      "[&_.markdown]:text-[15px]!",
      "[&_.markdown]:leading-[1.75]!",
      "[&_.markdown]:max-w-none",
    ]) {
      expect(cls).toContain(token);
    }
    expect(cls.join(" ")).not.toContain("max-w-[96ch]");
  });
});

describe("DetailActions 標題列插槽", () => {
  it("無 provider 時原地渲染", () => {
    render(
      <div data-testid="here">
        <DetailActions>
          <button type="button">分析</button>
        </DetailActions>
      </div>,
      { wrapper },
    );
    expect(within(screen.getByTestId("here")).getByRole("button", { name: "分析" })).toBeTruthy();
  });

  it("有 provider 時子節點出現在插槽節點內", () => {
    function Host() {
      const [slot, setSlot] = useState<HTMLDivElement | null>(null);
      return (
        <DetailActionsSlotProvider value={slot}>
          <header data-testid="bar">
            <DetailActionsSlot ref={setSlot} />
          </header>
          <main data-testid="page">
            <DetailActions>
              <button type="button">封存</button>
            </DetailActions>
          </main>
        </DetailActionsSlotProvider>
      );
    }
    render(<Host />, { wrapper });
    const slot = screen.getByTestId("bar").querySelector("[data-detail-actions]") as HTMLElement;
    expect(within(slot).getByRole("button", { name: "封存" })).toBeTruthy();
    expect(within(screen.getByTestId("page")).queryByRole("button", { name: "封存" })).toBeNull();
  });
});

describe("DetailIdentity 身分區", () => {
  it("關聯列以 data-relation-<mark> 定位；出身缺席段略過、其餘以「·」分隔", () => {
    const { container } = render(
      <DetailIdentity
        title="add-auth"
        copyLabel="複製名稱"
        provenance={[<span>Momo</span>, null, <span>3 天前</span>]}
        relations={[{ mark: "from", label: "來自", children: <span>login-flow</span> }]}
      />,
      { wrapper },
    );
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("add-auth");
    const from = container.querySelector("[data-relation-from]") as HTMLElement;
    expect(from.textContent).toBe("來自login-flow");
    const prov = container.querySelector("[data-identity-provenance]") as HTMLElement;
    expect(prov.textContent).toBe("Momo·3 天前");
  });

  it("關聯列標籤寬度隨字長：英文單字標籤不被固定寬度截斷、不折行", () => {
    const { container } = render(
      <DetailIdentity
        title="add-auth"
        copyLabel="Copy name"
        relations={[{ mark: "sibling", label: "Siblings", children: <span>sib-one</span> }]}
      />,
      { wrapper },
    );
    const label = (container.querySelector("[data-relation-sibling]") as HTMLElement).firstElementChild as HTMLElement;
    expect(label.textContent).toBe("Siblings");
    expect(label.className).not.toMatch(/(^|\s)w-8(\s|$)/);
    expect(label.className).toContain("min-w-8");
    expect(label.className).toContain("whitespace-nowrap");
  });
});
