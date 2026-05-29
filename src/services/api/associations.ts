import { invoke } from "@tauri-apps/api/core"
import type { Association } from "../../types/database"

export function getAssociations(): Promise<Association[]> {
  return invoke("get_associations")
}

export function createAssociation(name: string): Promise<Association> {
  return invoke("create_association", { name })
}

export function renameAssociation(id: number, name: string): Promise<void> {
  return invoke("rename_association", { id, name })
}

export function deleteAssociation(id: number): Promise<void> {
  return invoke("delete_association", { id })
}
