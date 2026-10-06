import { useCallback } from "react";
import { Link, useNavigate } from "react-router-dom";
import { PremiumGate } from "@operator/webcomponents";
import { useApiQuery } from "../api";
import { campaignsQuery, licenseQuery } from "../api/definitions";
import styles from "./CampaignsPage.module.css";

function CampaignList() {
  const query = useApiQuery(campaignsQuery());
  if (query.error) {
    return <output className={styles.status}>{query.error.message}</output>;
  }
  if (!query.data) {
    return <p className={styles.status}>Loading campaigns…</p>;
  }
  if (query.data.campaigns.length === 0) {
    return (
      <p className={styles.status}>
        No campaigns yet. Create one over MCP or with <code>POST /api/v1/campaigns</code>.
      </p>
    );
  }
  return (
    <ul className={styles.list}>
      {query.data.campaigns.map(({ campaign, members }) => (
        <li key={campaign.id} className={styles.card}>
          <div className={styles.heading}>
            <Link to={`/campaigns/${encodeURIComponent(campaign.id)}`}>{campaign.id}</Link>
            <span className={styles.state} data-status={campaign.status}>
              {campaign.status}
            </span>
          </div>
          <div>{campaign.title}</div>
          <div className={styles.muted}>
            {members} {members === 1 ? "ticket" : "tickets"} · created {campaign.created}
          </div>
        </li>
      ))}
    </ul>
  );
}

export function CampaignsPage() {
  const navigate = useNavigate();
  const license = useApiQuery(licenseQuery());
  const onAddLicense = useCallback(() => {
    void navigate("/settings/license");
  }, [navigate]);

  return (
    <main className={styles.page}>
      <h1 className={styles.heading}>
        Campaigns <span className={styles.badge}>Premium</span>
      </h1>
      <p>Epics of tickets, planned into dependency waves and launched together.</p>
      {license.error && <output className={styles.status}>{license.error.message}</output>}
      <PremiumGate feature="Campaigns" license={license.data} onAddLicense={onAddLicense}>
        <CampaignList />
      </PremiumGate>
    </main>
  );
}
