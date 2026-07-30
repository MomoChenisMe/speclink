; NSIS installer hooks（desktop-app spec「安裝 CLI 指令到 PATH」，design D5）：
; 安裝完成後把安裝目錄寫入使用者 PATH（HKCU\Environment）——CLI sidecar 與
; desktop 同目錄，寫入後新終端可直接執行 speclink；升級重跑安裝器即同版。
; 自含式實作：不依賴模板的 StrFunc／WinMessages include 狀態。

; StrStr（回傳自 needle 起的子字串；空字串＝未找到）。
Function SpeclinkStrStr
  Exch $R1 ; needle
  Exch
  Exch $R2 ; haystack
  Push $R3
  Push $R4
  Push $R5
  StrLen $R3 $R1
  StrCpy $R4 0
  speclink_strstr_loop:
    StrCpy $R5 $R2 $R3 $R4
    StrCmp $R5 $R1 speclink_strstr_done
    StrCmp $R5 "" speclink_strstr_done
    IntOp $R4 $R4 + 1
    Goto speclink_strstr_loop
  speclink_strstr_done:
  StrCpy $R1 $R5
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Exch $R1
FunctionEnd

!macro NSIS_HOOK_POSTINSTALL
  ReadRegStr $0 HKCU "Environment" "Path"
  Push $0
  Push "$INSTDIR"
  Call SpeclinkStrStr
  Pop $1
  ; 已在 PATH（含升級重跑）即不重複追加。
  StrCmp $1 "" speclink_add_path speclink_path_done
speclink_add_path:
  StrCmp $0 "" speclink_path_empty
  WriteRegExpandStr HKCU "Environment" "Path" "$0;$INSTDIR"
  Goto speclink_path_notify
speclink_path_empty:
  WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
speclink_path_notify:
  ; HWND_BROADCAST / WM_WININICHANGE 以數值自含（0xFFFF／0x1A）：通知既有程序
  ; 環境變數已變，新開終端即得新 PATH。
  SendMessage 0xFFFF 0x1A 0 "STR:Environment" /TIMEOUT=5000
speclink_path_done:
!macroend
