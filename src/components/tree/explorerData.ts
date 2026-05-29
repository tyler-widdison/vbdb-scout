import type { Association, Match, Season } from "../../types/database"
import type { NodeData } from "../../types/tree"
import * as api from "../../services/api"

export function associationToNode(association: Association): NodeData {
  return {
    id: association.id,
    name: association.name,
    type: "association",
    children: [],
    expanded: false,
    loaded: false,
  }
}

export function seasonToNode(season: Season, existing?: NodeData): NodeData {
  return {
    id: season.id,
    name: season.name,
    type: "season",
    parentId: season.association_id,
    children: existing?.children ?? [],
    expanded: existing?.expanded ?? false,
    loaded: existing?.loaded ?? false,
  }
}

export function matchToNode(match: Match): NodeData {
  return {
    id: match.id,
    name: match.name,
    type: "match",
    parentId: match.season_id,
    children: [],
    expanded: false,
    loaded: true,
  }
}

export async function loadNodeChildren(node: NodeData) {
  const previous = new Map<number, NodeData>()
  for (const child of node.children) previous.set(child.id, child)

  if (node.type === "association") {
    const seasons = await api.getSeasons(node.id)
    node.children = seasons.map((season) => seasonToNode(season, previous.get(season.id)))
    node.loaded = true
  } else if (node.type === "season") {
    const matches = await api.getMatches(node.id)
    node.children = matches.map(matchToNode)
    node.loaded = true
  }
}
