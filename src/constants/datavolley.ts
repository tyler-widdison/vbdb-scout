export type CodeOption = { value: string; label: string };

export const SKILLS: CodeOption[] = [
  { value: "S", label: "S" },
  { value: "R", label: "R" },
  { value: "A", label: "A" },
  { value: "B", label: "B" },
  { value: "D", label: "D" },
  { value: "E", label: "E" },
  { value: "F", label: "F" },
];

export const GRADES: CodeOption[] = [
  { value: "#", label: "#" },
  { value: "!", label: "!" },
  { value: "+", label: "+" },
  { value: "-", label: "-" },
  { value: "/", label: "/" },
  { value: "=", label: "=" },
];

export const ZONES: CodeOption[] = [
  { value: "1", label: "1" },
  { value: "2", label: "2" },
  { value: "3", label: "3" },
  { value: "4", label: "4" },
  { value: "5", label: "5" },
  { value: "6", label: "6" },
  { value: "7", label: "7" },
  { value: "8", label: "8" },
  { value: "9", label: "9" },
];

export const TEAMS: CodeOption[] = [
  { value: "*", label: "* Home" },
  { value: "a", label: "a Away" },
];

export const SERVE_SUB_TYPES: CodeOption[] = [
  { value: "H", label: "H" },
  { value: "T", label: "T" },
  { value: "M", label: "M" },
  { value: "N", label: "N" },
  { value: "Q", label: "Q" },
];

export const RECEPTION_SUB_TYPES: CodeOption[] = [
  { value: "M", label: "M" },
  { value: "R", label: "R" },
  { value: "L", label: "L" },
  { value: "W", label: "W" },
];

export const ATTACK_SUB_TYPES: CodeOption[] = [
  { value: "H", label: "H" },
  { value: "P", label: "P" },
  { value: "T", label: "T" },
];

export const BLOCK_SUB_TYPES: CodeOption[] = [];

export const DIG_SUB_TYPES: CodeOption[] = [];

export const ATTACK_TYPES: CodeOption[] = [];

export const SET_TYPES: CodeOption[] = [];

const SUB_TYPE_MAP: Record<string, CodeOption[]> = {
  S: SERVE_SUB_TYPES,
  R: RECEPTION_SUB_TYPES,
  A: ATTACK_SUB_TYPES,
  B: BLOCK_SUB_TYPES,
  D: DIG_SUB_TYPES,
  E: [],
  F: [],
};

export function getSubTypes(skill: string): CodeOption[] {
  return SUB_TYPE_MAP[skill.toUpperCase()] ?? [];
}

const SKILL_TYPE_MAP: Record<string, CodeOption[]> = {
  A: ATTACK_TYPES,
  S: SET_TYPES,
};

export function getSkillTypes(skill: string): CodeOption[] {
  return SKILL_TYPE_MAP[skill.toUpperCase()] ?? [];
}
