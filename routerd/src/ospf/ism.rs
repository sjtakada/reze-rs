//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF Interface State Machine
//        RFC2328 s9.1-9.3
//

use std::fmt;

/// Interface State.
pub enum IsmState {
    Down,
    Loopback,
    Waiting,
    PointToPoint,
    DROther,
    Backup,
    DR,
}

impl IsmState {
    pub fn to_string(&self) -> &str {
        match *self {
            IsmState::Down => "Down",
            IsmState::Loopback => "Loopback",
            IsmState::Waiting => "Waiting",
            IsmState::PointToPoint => "point-to-point",
            IsmState::DROther => "DR Other",
            IsmState::Backup => "Backup",
            IsmState::DR => "DR",
        }
    }
}

impl fmt::Display for IsmState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
}

impl fmt::Debug for IsmState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
} 

/// Events.
pub enum IsmEvent {
    InterfaceUp,
    WaitTimer,
    BackupSeen,
    NeighborChange,
    LoopInd,
    UnloopInd,
    InterfaceDown,
}

impl IsmEvent {

    pub fn to_string(&self) -> &str {
        match *self {
            IsmEvent::InterfaceUp => "InterfaceUp",
            IsmEvent::WaitTimer => "WaitTimer",
            IsmEvent::BackupSeen => "BackuSeen",
            IsmEvent::NeighborChange => "NeighborChange",
            IsmEvent::LoopInd => "LoopInd",
            IsmEvent::UnloopInd => "UnloopInd",
            IsmEvent::InterfaceDown => "InterfaceDown",
        }
    }
}

impl fmt::Display for IsmEvent {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
}

impl fmt::Debug for IsmEvent {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
}


/// Interface State Machine
pub struct OspfIsm {

    /// Interface State.
    state: IsmState,

    // TBD: link to ospf interface.
}

impl OspfIsm {

    /// Constructor.
    pub fn new() -> OspfIfsm {
        OspfIfsm {
            state: IsmState::InterfaceDown,
        }
    }

    /// InterfaceUp action.
    pub fn interface_up(&self) -> IsmState {
        let next_state = self.state;

        next_state
    }

    /// InterfaceDown action.
    pub fn interface_up(&self) -> IsmState {
        let next_state = self.state;

        next_state
    }

    /// BackupSeen action.
    pub fn backup_seen(&self) -> IsmState {
        let next_state = self.state;

        next_state
    }

    /// WaitTimer action.
    pub fn wait_timer(&self) -> IsmState {
        let next_state = self.state;

        next_state
    }

    /// NeighborChange action.
    pub fn neighbor_change(&self) -> IsmState {
        let next_state = self.state;

        next_state
    }

}
