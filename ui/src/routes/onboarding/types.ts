import type { Dispatch, SetStateAction } from "react";
import type { CollectionPreset } from "@operator/bindings/CollectionPreset";
import type { SessionWrapperType } from "@operator/bindings/SessionWrapperType";
import type { SetupExecutionTarget } from "@operator/bindings/SetupExecutionTarget";
import type { SetupStep } from "@operator/bindings/SetupStep";
import type {
  IntegrationCatalogEntryDto,
  SetupCollectionResponse,
  SetupStatusResponse,
} from "../../api-client";

export type WizardDraft = {
  configurationName: string;
  executionMode: "local" | "remote";
  premium: boolean;
  preset: CollectionPreset;
  taskFields: string[];
  wrapper: SessionWrapperType;
  executionTarget: SetupExecutionTarget;
  coderParameters: Array<{ id: number; name: string; value: string }>;
  useWorktrees: boolean;
  acceptanceCriteria: string;
  modelServers: string[];
  hostedCollectionIds: string[];
};

export type StepProps = {
  status: SetupStatusResponse;
  integrations: IntegrationCatalogEntryDto[];
  collections: SetupCollectionResponse[];
  /** The configuration does not exist yet, so `status` describes the default one. */
  creating: boolean;
  draft: WizardDraft;
  setDraft: Dispatch<SetStateAction<WizardDraft>>;
  exports: string[];
  addExport: (value: string) => void;
};

export type StepComponent = (props: StepProps) => React.JSX.Element;

/** One row of the setup sidebar. `number: null` marks an optional step this draft skips. */
export type StepRow = { slug: SetupStep; number: number | null; hint?: string };
