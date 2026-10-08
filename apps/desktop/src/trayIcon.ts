// 系統匣圖示資產：單色 template PNG（Speclink 標記剪影，可見形狀由 alpha 表達）。
// macOS 以 iconAsTemplate 依 alpha 渲染為系統色，自動適應深淺色選單列。前端擁有（design
// D1）故內嵌為 base64 常數，免資產管線與 fetch——tray.ts 以 Image.fromBytes 解碼（需 Rust
// image-png feature）。來源檔為 apps/desktop/src-tauri/icons/speclink-tray-18@2x.png
// （36×36，接合 S 單色剪影）；更新來源檔後須重新編碼此常數。
const TRAY_ICON_BASE64 =
  "iVBORw0KGgoAAAANSUhEUgAAACQAAAAkCAYAAADhAJiYAAAACXBIWXMAAAsTAAALEwEAmpwYAAABcklEQVRYhd2YPUoDQRSAPy2CCBKDir2W6bUQCwslFxCSKOQEnkE8RS4gQgoPoNYWuhtYbFVMJVaKEauoKwsvkGYzszPDY/WD1+58M+/tmx/4BywANc/wZh04Az6ANEAMfGS2gbdAIuPouspsAcPAMrfAfFlknoBVF5nNgPWSSrwDdReZ7C94CSwzAho4clJgoB8gAq6mxDmwiwdRgVm3UODRQuYLaKPEpUWaOihyaBC6QZlZwyp9a9XOJFUgNtRQU1tq0UIqS68qNaBfNqkVC6kkpyn2gCOgElqqIh/u5QyciFiedF8mpsq+dO88qTtguWxSCbCkLdU0pC+WtqJKSxpontQFMKMtFRm2oANNmY5swqlhlVRoG2poHA9atTOyPOB5nRr25Cg67agaW6RpMo5dZRoFZm0bz7J5F6Yu15eQMtn1asNFJmvz94FlPoEdF5k54LosMshjQEiZV3m0cGYQSGQInAJrBMD3USp72Pr7/AKjwMjhZxFjYwAAAABJRU5ErkJggg==";

/** 解碼內嵌 base64 為位元組——供 Image.fromBytes 建立 tray 圖示。 */
export function trayIconBytes(): Uint8Array {
  const binary = atob(TRAY_ICON_BASE64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
