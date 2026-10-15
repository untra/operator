import type { CapabilityDescription } from "../generated/CapabilityDescription";
import type { PermissionGrant } from "../generated/PermissionGrant";

const NO_DESCRIPTIONS: CapabilityDescription[] = [];

export interface PermissionSelectorProps {
  available: PermissionGrant;
  descriptions?: CapabilityDescription[];
  value: PermissionGrant;
  disabled?: boolean;
  onChange: (value: PermissionGrant) => void;
}

export function PermissionSelector({
  available,
  descriptions = NO_DESCRIPTIONS,
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
  const resources = [
    ...new Set(available.capabilities.map((capability) => capability.split(":")[0])),
  ];
  return (
    <fieldset disabled={disabled}>
      <legend>Key permissions</legend>
      <fieldset>
        <legend>Capabilities</legend>
        {resources.map((resource) => (
          <fieldset key={resource}>
            <legend>{resource}</legend>
            {available.capabilities
              .filter((capability) => capability.startsWith(`${resource}:`))
              .map((capability) => {
                const metadata = descriptions.find((entry) => entry.capability === capability);
                return (
                  <label key={capability}>
                    <input
                      type="checkbox"
                      checked={value.capabilities.includes(capability)}
                      onChange={() => toggle("capabilities", capability)}
                    />
                    {capability}
                    {metadata && (
                      <span>
                        {" "}
                        — {metadata.label}. {metadata.description}
                      </span>
                    )}
                  </label>
                );
              })}
          </fieldset>
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
