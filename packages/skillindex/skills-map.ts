// ── Barrel ─────────────────────────────────────────────────────
// skills-map.ts is intentionally just a barrel/aggregator — all catalog content lives under ./skills/.
export * from './skills/agents.ts';
export * from './skills/frontend.ts';
export { COMBO_SKILLS_MAP, SKILLS_MAP } from './skills/maps.ts';
export * from './skills/types.ts';
