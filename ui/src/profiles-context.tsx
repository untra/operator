import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { Outlet, useNavigate } from "react-router-dom";
import { OperatorApi, type ProfileSummary } from "./api-client";
import { AsyncState } from "@operator/webcomponents";
import { HostContext, useHost, type Host } from "./host";
import styles from "./profiles-context.module.css";

const SELECTED_PROFILE_KEY = "operator.selected-profile";

type ProfilesContextValue = {
  profiles: ProfileSummary[];
  selected: ProfileSummary | null;
  select: (id: string) => void;
  create: (name: string) => Promise<ProfileSummary>;
  refresh: () => Promise<void>;
};

const ProfilesContext = createContext<ProfilesContextValue | null>(null);

export function useProfiles(): ProfilesContextValue {
  const value = useContext(ProfilesContext);
  if (!value) {
    throw new Error("Configuration registry is unavailable");
  }
  return value;
}

// Remounted by profile ID so responses cannot update another configuration's views.
function ProfileScope({ profileId, children }: { profileId?: string; children: ReactNode }) {
  const serverHost = useHost();
  const host = useMemo<Host>(
    () => ({
      profileId,
      baseUrl: () => serverHost.baseUrl(),
      openExternal: (url) => serverHost.openExternal(url),
      browseFolder: () => serverHost.browseFolder(),
      openFile: (path) => serverHost.openFile(path),
    }),
    [profileId, serverHost],
  );
  return <HostContext.Provider value={host}>{children}</HostContext.Provider>;
}

export function ProfilesProvider() {
  const host = useHost();
  const api = useMemo(() => new OperatorApi(host), [host]);
  const [profiles, setProfiles] = useState<ProfileSummary[]>([]);
  const [selectedId, setSelectedId] = useState(() => localStorage.getItem(SELECTED_PROFILE_KEY));
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setProfiles(await api.profiles());
    setLoaded(true);
  }, [api]);

  useEffect(() => {
    let cancelled = false;
    api
      .profiles()
      .then((items) => {
        if (!cancelled) {
          setProfiles(items);
          setLoaded(true);
        }
        return undefined;
      })
      .catch((cause: unknown) => {
        if (!cancelled) {
          setError(cause instanceof Error ? cause.message : "Could not load configurations");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [api]);

  const selected =
    profiles.find((profile) => profile.id === selectedId) ??
    profiles.find((profile) => profile.is_default) ??
    profiles[0] ??
    null;

  const select = useCallback((id: string) => {
    localStorage.setItem(SELECTED_PROFILE_KEY, id);
    setSelectedId(id);
  }, []);

  const create = useCallback(
    async (name: string) => {
      await api.refreshCsrf();
      const profile = await api.createProfile(name);
      setProfiles((items) => [...items, profile]);
      select(profile.id);
      return profile;
    },
    [api, select],
  );

  const value = useMemo(
    () => ({ profiles, selected, select, create, refresh }),
    [profiles, selected, select, create, refresh],
  );

  if (error || !loaded) {
    return (
      <main className={styles.gate}>
        <AsyncState<null>
          value={
            error
              ? { status: "error", message: error }
              : { status: "loading", message: "Loading configurations…" }
          }
        >
          {() => null}
        </AsyncState>
      </main>
    );
  }
  return (
    <ProfilesContext.Provider value={value}>
      <ProfileScope key={selected?.id ?? "new"} profileId={selected?.id}>
        <Outlet />
      </ProfileScope>
    </ProfilesContext.Provider>
  );
}

export function ProfileSelector() {
  const { profiles, selected, select } = useProfiles();
  const navigate = useNavigate();

  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => {
      const profile = profiles.find((item) => item.id === event.target.value);
      if (!profile) {
        return;
      }
      select(profile.id);
      void navigate(profile.initialized ? "/" : "/onboarding");
    },
    [profiles, select, navigate],
  );

  const onCreate = useCallback(() => {
    void navigate("/onboarding?new=1");
  }, [navigate]);

  return (
    <div className={styles.selector}>
      <label htmlFor="configuration-selector">Configuration</label>
      <select id="configuration-selector" value={selected?.id ?? ""} onChange={onChange}>
        {profiles.map((profile) => (
          <option key={profile.id} value={profile.id}>
            {profile.name}
            {profile.initialized ? "" : " (draft)"}
          </option>
        ))}
      </select>
      <button type="button" onClick={onCreate}>
        New configuration
      </button>
    </div>
  );
}
