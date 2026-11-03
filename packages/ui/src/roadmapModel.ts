import type { ChangeItem } from "./adapter";
import { changeStage, planBlockedBy, planWave, type Stage } from "./stage";

/** 路線圖上的一站（spec desktop-roadmap「路線圖依波次排列變更」）。 */
export interface RoadmapStation {
  name: string;
  stage: Stage;
  completedTasks: number;
  totalTasks: number;
  blockedBy: string[];
  /** 站名等於 plan 的 next。 */
  next: boolean;
}

export interface RoadmapWave {
  wave: number;
  stations: RoadmapStation[];
}

/** 一條依賴線：前置（from）連到依賴它的站（to）。 */
export interface RoadmapEdge {
  from: string;
  to: string;
}

export interface RoadmapModel {
  waves: RoadmapWave[];
  edges: RoadmapEdge[];
  /** 沒有波次（metadata 無法解析）的變更名，只在看板顯示。 */
  unplaced: string[];
}

/** 線的鍵（`from→to`）：路線高亮與 SVG 疊層共用。 */
export function roadmapEdgeKey(edge: RoadmapEdge): string {
  return `${edge.from}→${edge.to}`;
}

/** 清單項（plan 的配置序）→ 波次欄、站、線與無法排程項；波內維持清單順序。 */
export function roadmapModel(changes: readonly ChangeItem[], next: string | null): RoadmapModel {
  const byWave = new Map<number, RoadmapStation[]>();
  const placed = new Set<string>();
  const unplaced: string[] = [];
  for (const change of changes) {
    const wave = planWave(change);
    if (wave === null) {
      unplaced.push(change.name);
      continue;
    }
    placed.add(change.name);
    const stations = byWave.get(wave) ?? [];
    stations.push({
      name: change.name,
      stage: changeStage(change),
      completedTasks: change.completedTasks,
      totalTasks: change.totalTasks,
      blockedBy: planBlockedBy(change),
      next: change.name === next,
    });
    byWave.set(wave, stations);
  }
  const edges = changes.flatMap((change) =>
    placed.has(change.name)
      ? (change.dependsOn ?? []).filter((dep) => placed.has(dep)).map((dep) => ({ from: dep, to: change.name }))
      : [],
  );
  const waves = [...byWave.entries()].sort(([a], [b]) => a - b).map(([wave, stations]) => ({ wave, stations }));
  return { waves, edges, unplaced };
}

/** 自 `start` 沿線的 `along` 端走到 `toward` 端，收集走得到的全部站（含 `start`）。 */
function reach(edges: readonly RoadmapEdge[], start: string, along: "from" | "to", toward: "from" | "to"): Set<string> {
  const seen = new Set([start]);
  const pending = [start];
  while (pending.length > 0) {
    const current = pending.pop()!;
    for (const edge of edges) {
      if (edge[along] === current && !seen.has(edge[toward])) {
        seen.add(edge[toward]);
        pending.push(edge[toward]);
      }
    }
  }
  return seen;
}

/** 停在 `name` 時亮起的路線：該站、遞移前置、遞移依賴者，以及兩端都在這組站內的線。 */
export function roadmapRoute(
  model: RoadmapModel,
  name: string,
): { stations: Set<string>; edges: Set<string> } {
  const before = reach(model.edges, name, "to", "from");
  const after = reach(model.edges, name, "from", "to");
  const stations = new Set([...before, ...after]);
  const edges = new Set(
    model.edges.filter((e) => stations.has(e.from) && stations.has(e.to)).map(roadmapEdgeKey),
  );
  return { stations, edges };
}
