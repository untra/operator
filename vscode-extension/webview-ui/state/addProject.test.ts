import { describe, expect, test } from "bun:test";
import { addProject } from "./addProject";
import type { ProjectSyncConfig } from "../../src/generated/ProjectSyncConfig";

const EXISTING_PROJECT: ProjectSyncConfig = {
  sync_user_id: "user-1",
  status_mapping: { todo: "To Do" },
  collection_name: "dev",
  type_mappings: { Bug: "FIX" },
  bidirectional: true,
};

describe("addProject", () => {
  test("keeps an existing project's configuration", () => {
    const projects = { PROJ: EXISTING_PROJECT };

    expect(addProject(projects, "PROJ")).toBe(projects);
    expect(projects.PROJ.collection_name).toBe("dev");
  });

  test("uses a dotted project key without treating it as a path", () => {
    const projects = addProject({}, "TEAM.PROJ");

    expect(Object.keys(projects)).toEqual(["TEAM.PROJ"]);
    expect(projects["TEAM.PROJ"]?.collection_name).toBeNull();
  });
});
