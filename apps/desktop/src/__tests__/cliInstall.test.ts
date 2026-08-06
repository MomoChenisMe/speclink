// CLI 佈署純邏輯（desktop-app spec「安裝 CLI 指令到 PATH」，design D5）：狀態
// 判定（未安裝／已安裝同版／版本不符）、平台分流的佈署計畫（macOS symlink、
// AppImage 複製、Windows 與 deb 僅回報）、AppImage 版本不符自我修復判定、
// 佈署目錄不在 PATH 的提示旗標。檔案系統操作歸 adapter，core 只做決策。
import { describe, expect, it, vi } from "vitest";

import {
  cliDeployPlan,
  cliInstallStatus,
  isDirOnPath,
  needsRedeploy,
  parseCliVersion,
} from "../core/cliInstall";
import type { CliInstallAdapter, CliInstallProbe } from "../adapter/cliInstall";
import { createAppStore } from "../store";

describe("CLI 佈署狀態判定", () => {
  it("未偵測到已佈署版本＝未安裝", () => {
    expect(cliInstallStatus(null, "0.2.0")).toEqual({ kind: "not-installed" });
  });

  it("已佈署且與 app 同版＝已安裝", () => {
    expect(cliInstallStatus("0.2.0", "0.2.0")).toEqual({ kind: "installed", version: "0.2.0" });
  });

  it("已佈署但與 app 版本不符＝版本不符", () => {
    expect(cliInstallStatus("0.1.0", "0.2.0")).toEqual({
      kind: "version-mismatch",
      version: "0.1.0",
    });
  });
});

describe("CLI --version 輸出解析", () => {
  it("speclink 0.1.0 (arm64) 解析出 0.1.0", () => {
    expect(parseCliVersion("speclink 0.1.0 (arm64)\n")).toBe("0.1.0");
  });

  it("帶 engine 版號的現行格式同樣解析出套件版號（instruction-downgrade-guard 的刻意變更）", () => {
    expect(parseCliVersion("speclink 0.1.0 (arm64, engine v1.14.0)\n")).toBe("0.1.0");
  });

  it("無法解析時回 null", () => {
    expect(parseCliVersion("garbage")).toBeNull();
  });
});

describe("佈署計畫平台分流", () => {
  const ctx = { home: "/Users/u", bundledCliPath: "/Applications/Speclink.app/Contents/MacOS/speclink" };

  it("macOS：於 ~/.local/bin 建 symlink 指向 app bundle 內 CLI", () => {
    expect(cliDeployPlan("macos", ctx)).toEqual({
      action: "symlink",
      linkPath: "/Users/u/.local/bin/speclink",
      targetPath: "/Applications/Speclink.app/Contents/MacOS/speclink",
    });
  });

  it("Linux AppImage：複製至 ~/.local/bin（掛載點隨執行變動、symlink 不可行）", () => {
    const linuxCtx = { home: "/home/u", bundledCliPath: "/tmp/.mount_abc/usr/bin/speclink" };
    expect(cliDeployPlan("linux-appimage", linuxCtx)).toEqual({
      action: "copy",
      destPath: "/home/u/.local/bin/speclink",
      sourcePath: "/tmp/.mount_abc/usr/bin/speclink",
    });
  });

  it("Windows：安裝器負責 PATH，app 內僅回報狀態", () => {
    expect(cliDeployPlan("windows", ctx)).toEqual({
      action: "none",
      reason: "installer-managed",
    });
  });

  it("Linux deb：包管理器佈署 /usr/bin，app 內僅回報狀態", () => {
    expect(cliDeployPlan("linux-deb", ctx)).toEqual({
      action: "none",
      reason: "package-managed",
    });
  });
});

describe("AppImage 啟動自我修復判定", () => {
  it("AppImage 且版本不符→需重佈署", () => {
    expect(needsRedeploy("linux-appimage", { kind: "version-mismatch", version: "0.1.0" })).toBe(
      true,
    );
  });

  it("AppImage 已安裝同版→不重佈署", () => {
    expect(needsRedeploy("linux-appimage", { kind: "installed", version: "0.2.0" })).toBe(false);
  });

  it("AppImage 未安裝→不主動佈署（安裝是使用者的顯式動作）", () => {
    expect(needsRedeploy("linux-appimage", { kind: "not-installed" })).toBe(false);
  });

  it("非 AppImage 平台不自我修復（macOS symlink 更新後自動指向新版）", () => {
    expect(needsRedeploy("macos", { kind: "version-mismatch", version: "0.1.0" })).toBe(false);
  });
});

describe("佈署目錄不在 PATH 的提示旗標", () => {
  it("目錄在 PATH 上→不提示", () => {
    expect(isDirOnPath("/Users/u/.local/bin", "/usr/bin:/Users/u/.local/bin", ":")).toBe(true);
  });

  it("目錄不在 PATH 上→提示加入方式", () => {
    expect(isDirOnPath("/Users/u/.local/bin", "/usr/bin:/usr/local/bin", ":")).toBe(false);
  });

  it("Windows 以分號分隔", () => {
    expect(isDirOnPath("C:\\Speclink", "C:\\Windows;C:\\Speclink", ";")).toBe(true);
  });
});

// --- store 接線（probe → core 判定 → 視圖；動作與 AppImage 自我修復） ---

function macProbe(over: Partial<CliInstallProbe> = {}): CliInstallProbe {
  return {
    platform: "macos",
    home: "/Users/u",
    pathEnv: "/usr/bin:/Users/u/.local/bin",
    pathDelimiter: ":",
    bundledCliPath: "/Applications/Speclink.app/Contents/MacOS/speclink",
    appVersion: "0.2.0",
    deployedVersionOutput: null,
    ...over,
  };
}

/** probe 依呼叫次序回傳 results（超出即重複末項）；deploy 可注入斷言。 */
function storeWith(results: CliInstallProbe[], deploy = vi.fn().mockResolvedValue(undefined)) {
  let calls = 0;
  const adapter: CliInstallAdapter = {
    probe: vi.fn().mockImplementation(() => {
      const probe = results[Math.min(calls, results.length - 1)];
      calls += 1;
      return Promise.resolve(probe);
    }),
    deploy,
  };
  return { store: createAppStore({ createSession: vi.fn() as never, cliInstall: adapter }), deploy };
}

describe("CLI 佈署 store 接線", () => {
  it("macOS 未安裝：探測後呈現未安裝且可佈署", async () => {
    const { store } = storeWith([macProbe()]);
    await store.getState().refreshCliInstall();

    expect(store.getState().cliInstall).toMatchObject({
      platform: "macos",
      status: { kind: "not-installed" },
      canDeploy: true,
      pathHint: false,
    });
  });

  it("installCli：對殼層送出 symlink 計畫並重探測為已安裝", async () => {
    const { store, deploy } = storeWith([
      macProbe(),
      macProbe({ deployedVersionOutput: "speclink 0.2.0 (arm64)\n" }),
    ]);
    await store.getState().refreshCliInstall();
    await store.getState().installCli();

    expect(deploy).toHaveBeenCalledWith({
      action: "symlink",
      linkPath: "/Users/u/.local/bin/speclink",
      targetPath: "/Applications/Speclink.app/Contents/MacOS/speclink",
    });
    expect(store.getState().cliInstall).toMatchObject({
      status: { kind: "installed", version: "0.2.0" },
      busy: false,
    });
  });

  it("已安裝但 ~/.local/bin 不在 PATH：pathHint 亮起", async () => {
    const { store } = storeWith([
      macProbe({
        pathEnv: "/usr/bin:/usr/local/bin",
        deployedVersionOutput: "speclink 0.2.0 (arm64)\n",
      }),
    ]);
    await store.getState().refreshCliInstall();

    expect(store.getState().cliInstall).toMatchObject({
      status: { kind: "installed", version: "0.2.0" },
      pathHint: true,
      deployDir: "/Users/u/.local/bin",
    });
  });

  it("AppImage 版本不符：refreshCliInstall 自動重佈署（啟動自我修復）", async () => {
    const appImage = macProbe({
      platform: "linux-appimage",
      home: "/home/u",
      bundledCliPath: "/tmp/.mount_abc/usr/bin/speclink",
      deployedVersionOutput: "speclink 0.1.0 (x86_64)\n",
    });
    const { store, deploy } = storeWith([
      appImage,
      { ...appImage, deployedVersionOutput: "speclink 0.2.0 (x86_64)\n" },
    ]);
    await store.getState().refreshCliInstall();

    expect(deploy).toHaveBeenCalledWith({
      action: "copy",
      destPath: "/home/u/.local/bin/speclink",
      sourcePath: "/tmp/.mount_abc/usr/bin/speclink",
    });
    expect(store.getState().cliInstall).toMatchObject({
      status: { kind: "installed", version: "0.2.0" },
    });
  });

  it("macOS 版本不符不自動重佈署（symlink 更新後自動指向新版，異常留給顯式動作）", async () => {
    const { store, deploy } = storeWith([
      macProbe({ deployedVersionOutput: "speclink 0.1.0 (arm64)\n" }),
    ]);
    await store.getState().refreshCliInstall();

    expect(deploy).not.toHaveBeenCalled();
    expect(store.getState().cliInstall).toMatchObject({
      status: { kind: "version-mismatch", version: "0.1.0" },
    });
  });

  it("Windows：僅回報狀態、不可佈署", async () => {
    const { store, deploy } = storeWith([
      macProbe({
        platform: "windows",
        home: "C:\\Users\\u",
        pathDelimiter: ";",
        pathEnv: "C:\\Windows",
        deployedVersionOutput: "speclink 0.2.0 (x86_64)\n",
      }),
    ]);
    await store.getState().refreshCliInstall();
    await store.getState().installCli();

    expect(deploy).not.toHaveBeenCalled();
    expect(store.getState().cliInstall).toMatchObject({
      status: { kind: "installed", version: "0.2.0" },
      canDeploy: false,
      pathHint: false,
    });
  });

  it("佈署失敗：浮出錯誤、解除 busy", async () => {
    const deploy = vi.fn().mockRejectedValue(new Error("permission denied"));
    const { store } = storeWith([macProbe()], deploy);
    await store.getState().refreshCliInstall();
    await store.getState().installCli();

    expect(store.getState().cliInstall).toMatchObject({
      busy: false,
      error: "permission denied",
    });
  });
});
