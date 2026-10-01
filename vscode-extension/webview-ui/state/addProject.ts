import type { ProjectSyncConfig } from "../../src/generated/ProjectSyncConfig";

export function addProject(
  projects: Record<string, ProjectSyncConfig>,
  key: string,
): Record<string, ProjectSyncConfig> {
  if (Object.hasOwn(projects, key)) {
    return projects;
  }

  return {
    ...projects,
    [key]: {
      sync_user_id: "",
      status_mapping: {},
      collection_name: null,
      type_mappings: {},
      bidirectional: false,
    },
  };
}
