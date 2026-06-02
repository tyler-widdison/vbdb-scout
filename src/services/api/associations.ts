import { Schema } from "effect"
import type { Association } from "../../types/database"
import { AssociationSchema } from "../effect/schemas"
import { tauri, tauriVoid } from "../effect/tauri"

export function getAssociations(): Promise<Association[]> {
  return tauri("get_associations", {}, Schema.Array(AssociationSchema)) as Promise<Association[]>
}

export function createAssociation(name: string): Promise<Association> {
  return tauri("create_association", { name }, AssociationSchema) as Promise<Association>
}

export function renameAssociation(id: number, name: string): Promise<void> {
  return tauriVoid("rename_association", { id, name })
}

export function deleteAssociation(id: number): Promise<void> {
  return tauriVoid("delete_association", { id })
}
