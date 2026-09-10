//! The lifecycle gate — the single adjudication point for station
//! transitions (platform architecture §15.1: the PM→RD handoff gate).
//!
//! Six closed stations: drafting → review → ready → applying → verified →
//! archived. The adjudication function is the only place a transition is
//! judged; an illegal transition is rejected with its reason. Local mode
//! gets a read-only station derivation (no approval/evidence semantics
//! exist locally, so review/ready/verified are never derived) — and this
//! change wires no existing verb through the gate: enforcement starts when
//! evidence and approvals land (順位 5).

/// The closed six-station lifecycle. Ordered: a legal transition advances
/// exactly one station; the derive helpers below map local change state
/// onto the stations that exist without approval/evidence semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LifecycleStation {
    Drafting,
    Review,
    Ready,
    Applying,
    Verified,
    Archived,
}

impl LifecycleStation {
    /// The legal path in order.
    const PATH: [LifecycleStation; 6] = [
        LifecycleStation::Drafting,
        LifecycleStation::Review,
        LifecycleStation::Ready,
        LifecycleStation::Applying,
        LifecycleStation::Verified,
        LifecycleStation::Archived,
    ];

    fn index(self) -> usize {
        Self::PATH.iter().position(|s| *s == self).expect("station is on the path")
    }
}

/// Why the gate refused a transition — the closed reason set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionRejection {
    /// The target lies ahead but intermediate stations were skipped;
    /// `missing` lists them in path order.
    SkippedStations { missing: Vec<LifecycleStation> },
    /// The lifecycle never moves backward or stays in place.
    NotForward {
        from: LifecycleStation,
        to: LifecycleStation,
    },
}

/// The single adjudication point: allow exactly one step forward along the
/// legal path, reject everything else with its reason. No verb is wired
/// through this gate yet — enforcement starts once approvals and evidence
/// exist (順位 5); the transition table itself is fixed here.
pub fn adjudicate_transition(
    from: LifecycleStation,
    to: LifecycleStation,
) -> Result<(), TransitionRejection> {
    let (fi, ti) = (from.index(), to.index());
    if ti <= fi {
        return Err(TransitionRejection::NotForward { from, to });
    }
    if ti - fi > 1 {
        return Err(TransitionRejection::SkippedStations {
            missing: LifecycleStation::PATH[fi + 1..ti].to_vec(),
        });
    }
    Ok(())
}

/// The three local change states a filesystem workspace can express.
/// Review/ready/verified need approval and evidence semantics that do not
/// exist locally, so they are never derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalChangeState {
    /// No started marker yet.
    NotStarted,
    /// The in-progress marker exists (started_at stamped).
    Started,
    /// The change sits in the archive.
    Archived,
}

/// Read-only local station derivation — a pure function over the observed
/// state: it reads nothing and writes nothing.
pub fn derive_local_station(state: LocalChangeState) -> LifecycleStation {
    match state {
        LocalChangeState::NotStarted => LifecycleStation::Drafting,
        LocalChangeState::Started => LifecycleStation::Applying,
        LocalChangeState::Archived => LifecycleStation::Archived,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- lifecycle gate 是單一裁決點 ---

    #[test]
    fn stations_are_a_closed_set_of_six() {
        let all = [
            LifecycleStation::Drafting,
            LifecycleStation::Review,
            LifecycleStation::Ready,
            LifecycleStation::Applying,
            LifecycleStation::Verified,
            LifecycleStation::Archived,
        ];
        for station in &all {
            match station {
                LifecycleStation::Drafting => {}
                LifecycleStation::Review => {}
                LifecycleStation::Ready => {}
                LifecycleStation::Applying => {}
                LifecycleStation::Verified => {}
                LifecycleStation::Archived => {}
            }
        }
        let unique: std::collections::BTreeSet<_> = all.into_iter().collect();
        assert_eq!(unique.len(), 6);
    }

    #[test]
    fn the_legal_path_is_allowed_step_by_step() {
        // 合法路徑 drafting→review→ready→applying→verified→archived
        // 逐步請求全數允許。
        let path = [
            LifecycleStation::Drafting,
            LifecycleStation::Review,
            LifecycleStation::Ready,
            LifecycleStation::Applying,
            LifecycleStation::Verified,
            LifecycleStation::Archived,
        ];
        for pair in path.windows(2) {
            assert_eq!(
                adjudicate_transition(pair[0], pair[1]),
                Ok(()),
                "step {:?} -> {:?} must be allowed",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn skipping_stations_rejects_and_names_the_missing_ones() {
        // drafting 直跳 verified：拒絕並指出缺少的中間站。
        let err = adjudicate_transition(LifecycleStation::Drafting, LifecycleStation::Verified)
            .expect_err("skipping stations must reject");
        assert_eq!(
            err,
            TransitionRejection::SkippedStations {
                missing: vec![
                    LifecycleStation::Review,
                    LifecycleStation::Ready,
                    LifecycleStation::Applying,
                ],
            }
        );
    }

    #[test]
    fn backward_and_in_place_transitions_reject() {
        // 生命週期不倒流：後退與原地都拒絕（fail closed，非跳站原因）。
        assert_eq!(
            adjudicate_transition(LifecycleStation::Review, LifecycleStation::Drafting),
            Err(TransitionRejection::NotForward {
                from: LifecycleStation::Review,
                to: LifecycleStation::Drafting,
            })
        );
        assert_eq!(
            adjudicate_transition(LifecycleStation::Applying, LifecycleStation::Applying),
            Err(TransitionRejection::NotForward {
                from: LifecycleStation::Applying,
                to: LifecycleStation::Applying,
            })
        );
    }

    #[test]
    fn local_states_derive_read_only_stations() {
        // 本地三態唯讀推導：未開工＝drafting、已標記開工＝applying、
        // 已封存＝archived。純函式——不讀不寫任何檔案。
        assert_eq!(
            derive_local_station(LocalChangeState::NotStarted),
            LifecycleStation::Drafting
        );
        assert_eq!(
            derive_local_station(LocalChangeState::Started),
            LifecycleStation::Applying
        );
        assert_eq!(
            derive_local_station(LocalChangeState::Archived),
            LifecycleStation::Archived
        );
    }
}
