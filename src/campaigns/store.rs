//! Campaign files under `<tickets>/campaigns/`. Membership lives on tickets.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::model::{Campaign, CampaignStatus, CampaignWorkspace};
use crate::config::Config;
use crate::queue::{ids, Ticket};

/// Fields a new campaign starts with.
#[derive(Debug, Clone, Default)]
pub struct NewCampaign {
    pub title: String,
    pub goal: Option<String>,
    pub governance: Option<String>,
    pub governance_overrides: Option<crate::config::GovernancePolicy>,
    pub workspace: Option<CampaignWorkspace>,
    pub external_ref: Option<String>,
}

/// Reads and writes campaign files.
pub struct CampaignStore {
    dir: PathBuf,
}

impl CampaignStore {
    pub fn new(config: &Config) -> Self {
        Self::at(config.tickets_path().join("campaigns"))
    }

    pub fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Every campaign with its file path, ordered by id.
    fn entries(&self) -> Vec<(PathBuf, Campaign)> {
        let mut entries: Vec<(PathBuf, Campaign)> = fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
            .filter_map(|p| Some((p.clone(), read_campaign(&p).ok()?)))
            .collect();
        entries.sort_by(|a, b| a.1.id.cmp(&b.1.id));
        entries
    }

    pub fn list(&self) -> Vec<Campaign> {
        self.entries().into_iter().map(|(_, c)| c).collect()
    }

    pub fn get(&self, id: &str) -> Option<Campaign> {
        self.list().into_iter().find(|c| c.id == id)
    }

    pub fn find_by_external_ref(&self, external_ref: &str) -> Option<Campaign> {
        self.list()
            .into_iter()
            .find(|c| c.external_ref.as_deref() == Some(external_ref))
    }

    pub fn create(&self, new: NewCampaign) -> Result<Campaign> {
        let existing: Vec<String> = self.list().into_iter().map(|c| c.id).collect();
        let id = ids::next_ticket_id("CAMP", existing.iter().map(String::as_str));
        let body = format!(
            "# {}\n\n{}\n",
            new.title,
            new.goal.as_deref().unwrap_or_default()
        );
        let campaign = Campaign {
            id,
            title: new.title,
            status: CampaignStatus::Draft,
            created: chrono::Local::now().format("%Y-%m-%d").to_string(),
            governance: new.governance,
            governance_overrides: new.governance_overrides,
            workspace: new.workspace,
            external_ref: new.external_ref,
            body,
        };
        fs::create_dir_all(&self.dir).context("Failed to create campaigns directory")?;
        let slug = ids::slugify(&campaign.title, 50);
        let path = ids::unique_path(&self.dir, &format!("{}-{slug}", campaign.id));
        write_campaign(&path, &campaign)?;
        Ok(campaign)
    }

    pub fn set_status(&self, id: &str, status: CampaignStatus) -> Result<Campaign> {
        let (path, mut campaign) = self
            .entries()
            .into_iter()
            .find(|(_, c)| c.id == id)
            .with_context(|| format!("campaign {id} not found"))?;
        campaign.status = status;
        write_campaign(&path, &campaign)?;
        Ok(campaign)
    }
}

/// Tickets that belong to `campaign_id`.
pub fn members<'a>(campaign_id: &str, tickets: &'a [Ticket]) -> Vec<&'a Ticket> {
    tickets
        .iter()
        .filter(|t| t.campaign.as_deref() == Some(campaign_id))
        .collect()
}

fn read_campaign(path: &Path) -> Result<Campaign> {
    let content = fs::read_to_string(path)?;
    let after_open = content
        .trim_start()
        .strip_prefix("---")
        .context("campaign file has no frontmatter")?;
    let end = after_open
        .find("\n---")
        .context("campaign frontmatter is not closed")?;
    let mut campaign: Campaign = serde_saphyr::from_str(&after_open[..end])?;
    campaign.body = after_open[end + 4..].trim_start_matches('\n').to_string();
    Ok(campaign)
}

fn write_campaign(path: &Path, campaign: &Campaign) -> Result<()> {
    let yaml = serde_saphyr::to_string(campaign).context("Failed to serialize campaign")?;
    fs::write(path, format!("---\n{yaml}---\n\n{}", campaign.body))
        .context("Failed to write campaign file")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(dir: &Path) -> CampaignStore {
        CampaignStore::at(dir.join("campaigns"))
    }

    #[test]
    fn test_create_then_get_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        let created = store
            .create(NewCampaign {
                title: "Pricing overhaul".into(),
                goal: Some("Ship v2 pricing.".into()),
                workspace: Some(CampaignWorkspace::Docker {
                    image: "untra/operator:latest".into(),
                }),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(created.id, "CAMP-0001");
        let read = store.get("CAMP-0001").unwrap();
        assert_eq!(read, created);
        assert!(read.body.contains("Ship v2 pricing."));
        assert!(dir
            .path()
            .join("campaigns/CAMP-0001-pricing-overhaul.md")
            .exists());
    }

    #[test]
    fn test_ids_are_sequential_and_titles_may_repeat() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        for _ in 0..3 {
            store
                .create(NewCampaign {
                    title: "Same".into(),
                    ..Default::default()
                })
                .unwrap();
        }
        let ids: Vec<String> = store.list().into_iter().map(|c| c.id).collect();
        assert_eq!(ids, vec!["CAMP-0001", "CAMP-0002", "CAMP-0003"]);
    }

    #[test]
    fn test_set_status_persists_and_keeps_body() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        store
            .create(NewCampaign {
                title: "Run".into(),
                goal: Some("Goal text.".into()),
                ..Default::default()
            })
            .unwrap();
        store
            .set_status("CAMP-0001", CampaignStatus::Running)
            .unwrap();
        let read = store.get("CAMP-0001").unwrap();
        assert_eq!(read.status, CampaignStatus::Running);
        assert!(read.body.contains("Goal text."));
        assert!(store
            .set_status("CAMP-0009", CampaignStatus::Paused)
            .is_err());
    }

    #[test]
    fn test_find_by_external_ref() {
        let dir = tempfile::tempdir().unwrap();
        let store = store(dir.path());
        store
            .create(NewCampaign {
                title: "Epic".into(),
                external_ref: Some("jira:PROJ-1".into()),
                ..Default::default()
            })
            .unwrap();
        let found = store.find_by_external_ref("jira:PROJ-1").unwrap();
        assert_eq!(found.id, "CAMP-0001");
        assert_eq!(found.status, CampaignStatus::Draft);
        assert!(store.find_by_external_ref("jira:PROJ-2").is_none());
    }

    #[test]
    fn test_assign_and_members() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("20260101-1200-FEAT-demo-a.md");
        fs::write(
            &path,
            "---\nid: FEAT-0001\nstatus: queued\n---\n\n# Feature: a\n",
        )
        .unwrap();
        let mut ticket = Ticket::from_file(&path).unwrap();
        ticket.set_campaign(Some("CAMP-0001")).unwrap();
        let tickets = vec![Ticket::from_file(&path).unwrap()];
        assert_eq!(members("CAMP-0001", &tickets).len(), 1);

        ticket.set_campaign(None).unwrap();
        let reread = Ticket::from_file(&path).unwrap();
        assert!(reread.campaign.is_none());
        assert!(!reread.content.contains("campaign"));
    }
}
