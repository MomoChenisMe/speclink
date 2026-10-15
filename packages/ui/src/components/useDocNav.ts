import { useCallback, useRef, useState } from "react";

import type { DecisionJump } from "./DesignDoc";

/**
 * 變更文件之間的跳轉狀態（變更詳情頁與已封存頁共用；design D5／D11／D14）：提案能力籤→規格段、
 * 任務 D 籤→設計決策卡、手動提示框「前往」→任務列。每個請求由目標元件處理完即清；`setTab` 切到目標文件。
 * 決策卡與任務列的請求帶遞增 seq，同一個目標可重複跳。
 */
export function useDocNav(setTab: (tab: string) => void) {
  // 規格段選定的能力：「開啟 ▾」指向它。
  const [specCap, setSpecCap] = useState<string | null>(null);
  // 能力籤的捲動請求：該規格段捲到後即清——自文件樹切回規格文件時自頂端呈現。
  const [specJump, setSpecJump] = useState<{ cap: string } | null>(null);
  const [designJump, setDesignJump] = useState<DecisionJump | null>(null);
  const [taskLocate, setTaskLocate] = useState<{ ordinal: number; seq: number } | null>(null);
  const seq = useRef(0);
  const clearSpecJump = useCallback(() => setSpecJump(null), []);
  const clearDesignJump = useCallback(() => setDesignJump(null), []);
  const clearTaskLocate = useCallback(() => setTaskLocate(null), []);
  return {
    specCap,
    specJump,
    designJump,
    taskLocate,
    clearSpecJump,
    clearDesignJump,
    clearTaskLocate,
    openSpec: (cap: string) => {
      setSpecCap(cap);
      setSpecJump({ cap });
      setTab("specs");
    },
    jumpDecision: (id: string) => {
      setDesignJump({ id, seq: ++seq.current });
      setTab("design");
    },
    locateTask: (ordinal: number) => setTaskLocate({ ordinal, seq: ++seq.current }),
    /** 換變更時清掉選定的能力與在途請求。 */
    reset: () => {
      setSpecCap(null);
      setSpecJump(null);
      setDesignJump(null);
      setTaskLocate(null);
    },
  };
}
