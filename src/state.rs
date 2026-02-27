use std::collections::{BTreeMap, BTreeSet};

use rand::seq::SliceRandom as _;
use tokio::sync::Mutex;

pub struct ApplicationState {
    key: String,
    templates: tera::Tera,
    synchronized: Mutex<SynchronizedState>,
}

#[derive(Default)]
struct SynchronizedState {
    candidates: CandidateList,
    ballots: BTreeMap<String, Vec<String>>,
}

impl ApplicationState {
    pub fn new(key: String) -> Self {
        Self {
            key,
            templates: tera::Tera::new("assets/**/*").unwrap(),
            synchronized: Default::default(),
        }
    }

    pub fn templates(&self) -> &tera::Tera {
        &self.templates
    }

    pub fn check_key(&self, key: &str) -> bool {
        key == self.key
    }

    pub async fn list_candidates(&self) -> CandidateList {
        self.synchronized.lock().await.candidates.clone()
    }

    pub async fn list_ballots(&self) -> BTreeSet<String> {
        self.synchronized
            .lock()
            .await
            .ballots
            .keys()
            .cloned()
            .collect()
    }

    pub async fn add_ballot(
        &self,
        callsign: String,
        ranking: Vec<String>,
    ) -> Result<(), &'static str> {
        let mut state = self.synchronized.lock().await;
        if !ranking.iter().all(|c| state.candidates.contains(c)) {
            return Err("invalid_candidate");
        }
        match state.ballots.entry(callsign) {
            std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(ranking);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => Err("duplicate_ballot"),
        }
    }

    pub async fn set_candidates(&self, new_candidates: BTreeSet<String>, allow_leave_empty: bool) {
        let mut state = self.synchronized.lock().await;
        state.candidates = CandidateList {
            candidates: new_candidates,
            allow_leave_empty,
        };
        state.ballots.clear();
    }

    pub async fn take_data(&self) -> Data {
        let mut state = self.synchronized.lock().await;
        let candidates = std::mem::take(&mut state.candidates);
        let ballots = std::mem::take(&mut state.ballots);
        drop(state);

        let (people, mut ballots): (_, Vec<_>) = ballots.into_iter().unzip();

        // prevent ballots from being in the same order as people
        ballots.shuffle(&mut rand::rng());

        Data {
            candidates,
            people,
            ballots,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CandidateList {
    pub candidates: BTreeSet<String>,
    pub allow_leave_empty: bool,
}

impl CandidateList {
    pub fn contains(&self, s: &str) -> bool {
        if self.allow_leave_empty && s == "LEAVEEMPTY" {
            true
        } else {
            self.candidates.contains(s)
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Data {
    pub candidates: CandidateList,
    pub people: Vec<String>,
    pub ballots: Vec<Vec<String>>,
}
