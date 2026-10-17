/** Device-local appearance only; project files and Remote policy are untouched. */
export const PROJECT_COLORS = ["teal", "blue", "purple", "pink", "orange", "green"] as const;
export type ProjectColor = typeof PROJECT_COLORS[number];
export type ProjectColors = Record<string, ProjectColor>;
const STORAGE_KEY = "speclink.projectColors";
export const isProjectColor = (value: unknown): value is ProjectColor =>
  typeof value === "string" && PROJECT_COLORS.some((color) => color === value);

export function readProjectColors(storage?: Storage): ProjectColors {
  try {
    const value: unknown = JSON.parse((storage ?? localStorage).getItem(STORAGE_KEY) ?? "null");
    if (!value || typeof value !== "object" || Array.isArray(value)) return {};
    return Object.fromEntries(Object.entries(value).filter(([, color]) => isProjectColor(color))) as ProjectColors;
  } catch { return {}; }
}

/** Let the settings boundary report storage failures before adopting a new color. */
export function writeProjectColors(colors: ProjectColors, storage: Storage = localStorage): void {
  storage.setItem(STORAGE_KEY, JSON.stringify(colors));
}
