/**
 * Hooks for fetching and managing steps.
 */
import { useState, useEffect, useCallback } from 'react';
import { useApi } from '@backstage/core-plugin-api';
import { operatorApiRef } from '../api';
import type { StepResponse, UpdateStepRequest } from '../api/types';

/** Hook to fetch steps for an issue type */
export function useSteps(issueTypeKey: string) {
  const api = useApi(operatorApiRef);
  const [steps, setSteps] = useState<StepResponse[]>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      const data = await api.getSteps(issueTypeKey);
      setSteps(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setLoading(false);
    }
  }, [api, issueTypeKey]);

  useEffect(() => {
    load();
  }, [load]);

  return {
    steps,
    loading,
    error,
    retry: load,
  };
}

/** Hook to fetch a single step */
export function useStep(issueTypeKey: string, stepName: string) {
  const api = useApi(operatorApiRef);
  const [step, setStep] = useState<StepResponse>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      const data = await api.getStep(issueTypeKey, stepName);
      setStep(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setLoading(false);
    }
  }, [api, issueTypeKey, stepName]);

  useEffect(() => {
    load();
  }, [load]);

  return {
    step,
    loading,
    error,
    retry: load,
  };
}

/** Hook to update a step */
export function useUpdateStep() {
  const api = useApi(operatorApiRef);
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<Error>();
  const [updatedStep, setUpdatedStep] = useState<StepResponse>();

  const updateStep = useCallback(
    async (
      issueTypeKey: string,
      stepName: string,
      request: UpdateStepRequest,
    ) => {
      setUpdating(true);
      setError(undefined);
      try {
        const result = await api.updateStep(issueTypeKey, stepName, request);
        setUpdatedStep(result);
        return result;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setUpdating(false);
      }
    },
    [api],
  );

  return {
    updateStep,
    updating,
    error,
    updatedStep,
  };
}
