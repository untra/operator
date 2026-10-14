import type { PermissionGrant } from "../generated/PermissionGrant";

export interface PermissionSelectorProps {
  available: PermissionGrant;
  value: PermissionGrant;
  disabled?: boolean;
  onChange: (value: PermissionGrant) => void;
}

export function PermissionSelector({
  available,
  value,
  disabled,
  onChange,
}: PermissionSelectorProps) {
  function toggle(field: "capabilities" | "projects", item: string) {
    const selected = value[field];
    onChange({
      ...value,
      [field]: selected.includes(item)
        ? selected.filter((entry) => entry !== item)
        : [...selected, item],
    });
  }
  return (
    <fieldset disabled={disabled}>
      <legend>Key permissions</legend>
      <fieldset>
        <legend>Capabilities</legend>
        {available.capabilities.map((capability) => (
          <label key={capability}>
            <input
              type="checkbox"
              checked={value.capabilities.includes(capability)}
              onChange={() => toggle("capabilities", capability)}
            />
            {capability}
          </label>
        ))}
      </fieldset>
      <fieldset>
        <legend>Projects</legend>
        <p>No selected projects means no project access.</p>
        {available.projects.map((project) => (
          <label key={project}>
            <input
              type="checkbox"
              checked={value.projects.includes(project)}
              onChange={() => toggle("projects", project)}
            />
            {project}
          </label>
        ))}
      </fieldset>
      {available.instance_wide && (
        <label>
          <input
            type="checkbox"
            checked={value.instance_wide}
            onChange={(event) => onChange({ ...value, instance_wide: event.target.checked })}
          />
          Allow selected instance operations
        </label>
      )}
    </fieldset>
  );
}
