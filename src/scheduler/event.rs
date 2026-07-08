//! Angle-domain events and their armed (timestamped) form.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventId {
    /// Injector opens.
    InjOpen(u8),
    /// Injector closes.
    InjClose(u8),
    /// Coil dwell begins (charge).
    DwellStart(u8),
    /// Spark: coil fires.
    Fire(u8),
}

/// An event pinned to a crank-cycle angle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AngleEvent {
    pub id: EventId,
    /// Degrees within the engine cycle, `0..cycle_deg`.
    pub angle_deg: f32,
}

/// An armed event: fire `id` at absolute time `t_us`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Armed {
    pub id: EventId,
    pub t_us: u64,
}
