use serde::{Deserialize, Serialize};

use crate::{MovementJourneyPhase, MpsError, MpsResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseAllocation {
    pub phase: MovementJourneyPhase,
    pub minutes: u32,
}

/// Returns the phase allocation template for a supported duration (45, 60, 75 minutes).
/// Returns `MpsError::UnsupportedDuration` for any other value.
pub fn phase_allocations_for_duration(duration_minutes: u32) -> MpsResult<Vec<PhaseAllocation>> {
    use MovementJourneyPhase::*;

    let allocations = match duration_minutes {
        45 => vec![
            PhaseAllocation {
                phase: Arrive,
                minutes: 4,
            },
            PhaseAllocation {
                phase: Prepare,
                minutes: 7,
            },
            PhaseAllocation {
                phase: Build,
                minutes: 15,
            },
            PhaseAllocation {
                phase: Integrate,
                minutes: 7,
            },
            PhaseAllocation {
                phase: Challenge,
                minutes: 6,
            },
            PhaseAllocation {
                phase: Transfer,
                minutes: 3,
            },
            PhaseAllocation {
                phase: ResetRetest,
                minutes: 3,
            },
        ],
        60 => vec![
            PhaseAllocation {
                phase: Arrive,
                minutes: 5,
            },
            PhaseAllocation {
                phase: Prepare,
                minutes: 10,
            },
            PhaseAllocation {
                phase: Build,
                minutes: 20,
            },
            PhaseAllocation {
                phase: Integrate,
                minutes: 10,
            },
            PhaseAllocation {
                phase: Challenge,
                minutes: 8,
            },
            PhaseAllocation {
                phase: Transfer,
                minutes: 4,
            },
            PhaseAllocation {
                phase: ResetRetest,
                minutes: 3,
            },
        ],
        75 => vec![
            PhaseAllocation {
                phase: Arrive,
                minutes: 6,
            },
            PhaseAllocation {
                phase: Prepare,
                minutes: 12,
            },
            PhaseAllocation {
                phase: Build,
                minutes: 26,
            },
            PhaseAllocation {
                phase: Integrate,
                minutes: 13,
            },
            PhaseAllocation {
                phase: Challenge,
                minutes: 10,
            },
            PhaseAllocation {
                phase: Transfer,
                minutes: 5,
            },
            PhaseAllocation {
                phase: ResetRetest,
                minutes: 3,
            },
        ],
        _ => {
            return Err(MpsError::UnsupportedDuration {
                requested: duration_minutes,
            })
        }
    };

    Ok(allocations)
}

/// Returns the total minutes across all phase allocations.
pub fn total_minutes(allocations: &[PhaseAllocation]) -> u32 {
    allocations.iter().map(|a| a.minutes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use MovementJourneyPhase::*;

    #[test]
    fn allocations_45_minutes_exact_values() {
        let allocs = phase_allocations_for_duration(45).unwrap();
        let expected = [
            (Arrive, 4),
            (Prepare, 7),
            (Build, 15),
            (Integrate, 7),
            (Challenge, 6),
            (Transfer, 3),
            (ResetRetest, 3),
        ];
        assert_eq!(allocs.len(), expected.len());
        for (alloc, (phase, minutes)) in allocs.iter().zip(expected.iter()) {
            assert_eq!(alloc.phase, *phase);
            assert_eq!(alloc.minutes, *minutes);
        }
        assert_eq!(total_minutes(&allocs), 45);
    }

    #[test]
    fn allocations_60_minutes_exact_values() {
        let allocs = phase_allocations_for_duration(60).unwrap();
        let expected = [
            (Arrive, 5),
            (Prepare, 10),
            (Build, 20),
            (Integrate, 10),
            (Challenge, 8),
            (Transfer, 4),
            (ResetRetest, 3),
        ];
        assert_eq!(allocs.len(), expected.len());
        for (alloc, (phase, minutes)) in allocs.iter().zip(expected.iter()) {
            assert_eq!(alloc.phase, *phase);
            assert_eq!(alloc.minutes, *minutes);
        }
        assert_eq!(total_minutes(&allocs), 60);
    }

    #[test]
    fn allocations_75_minutes_exact_values() {
        let allocs = phase_allocations_for_duration(75).unwrap();
        let expected = [
            (Arrive, 6),
            (Prepare, 12),
            (Build, 26),
            (Integrate, 13),
            (Challenge, 10),
            (Transfer, 5),
            (ResetRetest, 3),
        ];
        assert_eq!(allocs.len(), expected.len());
        for (alloc, (phase, minutes)) in allocs.iter().zip(expected.iter()) {
            assert_eq!(alloc.phase, *phase);
            assert_eq!(alloc.minutes, *minutes);
        }
        assert_eq!(total_minutes(&allocs), 75);
    }

    #[test]
    fn unsupported_duration_50_returns_error() {
        let err = phase_allocations_for_duration(50).unwrap_err();
        assert_eq!(err, MpsError::UnsupportedDuration { requested: 50 });
    }

    #[test]
    fn phase_order_is_correct() {
        let allocs = phase_allocations_for_duration(60).unwrap();
        let expected_order = [
            Arrive,
            Prepare,
            Build,
            Integrate,
            Challenge,
            Transfer,
            ResetRetest,
        ];
        let actual_order: Vec<_> = allocs.iter().map(|a| a.phase).collect();
        assert_eq!(actual_order, expected_order);
    }
}
