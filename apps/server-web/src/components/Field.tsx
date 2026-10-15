import type { HTMLInputTypeAttribute } from "react";
import { Input, Label } from "@speclink/ui";

// 一致的表單欄位：label ＋ input ＋ 選配的灰字說明 ＋ 鄰近欄位且以 role=alert 宣告的錯誤（D6）。
// setup、invite 與登入等專注流程表單共用，避免重複 label／aria 接線。
export function Field({
  id,
  label,
  value,
  onChange,
  type = "text",
  autoComplete,
  hint,
  error,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: HTMLInputTypeAttribute;
  autoComplete?: string;
  hint?: string;
  error?: string;
}) {
  const describedBy = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(" ") || undefined;
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        type={type}
        autoComplete={autoComplete}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        aria-invalid={Boolean(error)}
        aria-describedby={describedBy}
      />
      {hint && (
        <p id={`${id}-hint`} className="text-xs text-muted-foreground">
          {hint}
        </p>
      )}
      {error && (
        <p id={`${id}-error`} role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
