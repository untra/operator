import type { Dispatch, SetStateAction } from 'react';
import type { CollectionPreset } from '@operator/bindings/CollectionPreset';
import type { SessionWrapperType } from '@operator/bindings/SessionWrapperType';
import type { SetupExecutionTarget } from '@operator/bindings/SetupExecutionTarget';
import type {
  IntegrationCatalogEntryDto,
  OperatorApi,
  SetupCollectionResponse,
  SetupStatusResponse,
} from '../../api-client';

export type WizardDraft = {
  preset: CollectionPreset;
  taskFields: string[];
  wrapper: SessionWrapperType;
  executionTarget: SetupExecutionTarget;
  useWorktrees: boolean;
  acceptanceCriteria: string;
  modelServers: string[];
  hostedCollectionIds: string[];
};

export type StepProps = {
  api: OperatorApi;
  status: SetupStatusResponse;
  integrations: IntegrationCatalogEntryDto[];
  collections: SetupCollectionResponse[];
  draft: WizardDraft;
  setDraft: Dispatch<SetStateAction<WizardDraft>>;
  exports: string[];
  addExport: (value: string) => void;
};

export type StepComponent = (props: StepProps) => React.JSX.Element;
