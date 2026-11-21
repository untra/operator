import { afterEach, expect, jest, test } from "bun:test";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { PermissionSelector } from "../../../webcomponents/src/components/PermissionSelector";
import type { PermissionGrant } from "@operator/bindings/PermissionGrant";

const AVAILABLE: PermissionGrant = {
  profile_id: "00000000-0000-0000-0000-000000000001",
  instance_wide: true,
  capabilities: ["tickets:read", "tickets:create"],
  projects: ["operator"],
};
const EMPTY: PermissionGrant = {
  ...AVAILABLE,
  instance_wide: false,
  capabilities: [],
  projects: [],
};
afterEach(cleanup);

test("selecting a capability does not implicitly grant projects or instance operations", () => {
  const onChange = jest.fn();
  render(<PermissionSelector available={AVAILABLE} value={EMPTY} onChange={onChange} />);
  fireEvent.click(screen.getByLabelText("tickets:read"));
  expect(onChange).toHaveBeenCalledWith({ ...EMPTY, capabilities: ["tickets:read"] });
});

test("project removal preserves the profile and selected capabilities", () => {
  const onChange = jest.fn();
  render(<PermissionSelector available={AVAILABLE} value={AVAILABLE} onChange={onChange} />);
  fireEvent.click(screen.getByLabelText("operator"));
  expect(onChange).toHaveBeenCalledWith({ ...AVAILABLE, projects: [] });
});
