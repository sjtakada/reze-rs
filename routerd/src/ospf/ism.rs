//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF Interface State Machine
//        RFC2328 s9.1-9.3
//

use std::fmt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use log::debug;

use super::interface::OspfInterface;

/// Interface State.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

type IsmTuple = (IsmState, IsmEvent);

/// Interface State Machine
pub struct OspfIsm {

    actions: HashMap<IsmTuple, &'static dyn Fn(&mut OspfInterface) -> IsmState>,
}

impl OspfIsm {

    /// Constructor.
    pub fn new() -> OspfIsm {
        OspfIsm {
            actions: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.actions.insert((IsmState::Down, IsmEvent::InterfaceUp), &OspfIsm::interface_up);
        self.actions.insert((IsmState::Down, IsmEvent::LoopInd), &OspfIsm::loop_ind);

        self.actions.insert((IsmState::Loopback, IsmEvent::UnloopInd), &OspfIsm::unloop_ind);
        self.actions.insert((IsmState::Loopback, IsmEvent::InterfaceDown), &OspfIsm::interface_down);

        self.actions.insert((IsmState::Waiting, IsmEvent::WaitTimer), &OspfIsm::wait_timer);
        self.actions.insert((IsmState::Waiting, IsmEvent::BackupSeen), &OspfIsm::backup_seen);
        self.actions.insert((IsmState::Waiting, IsmEvent::LoopInd), &OspfIsm::loop_ind);
        self.actions.insert((IsmState::Waiting, IsmEvent::InterfaceDown), &OspfIsm::interface_down);

        self.actions.insert((IsmState::PointToPoint, IsmEvent::LoopInd), &OspfIsm::loop_ind);
        self.actions.insert((IsmState::PointToPoint, IsmEvent::InterfaceDown), &OspfIsm::interface_down);

        self.actions.insert((IsmState::DROther, IsmEvent::NeighborChange), &OspfIsm::neighbor_change);
        self.actions.insert((IsmState::DROther, IsmEvent::LoopInd), &OspfIsm::loop_ind);
        self.actions.insert((IsmState::DROther, IsmEvent::InterfaceDown), &OspfIsm::interface_down);

        self.actions.insert((IsmState::Backup, IsmEvent::NeighborChange), &OspfIsm::neighbor_change);
        self.actions.insert((IsmState::Backup, IsmEvent::LoopInd), &OspfIsm::loop_ind);
        self.actions.insert((IsmState::Backup, IsmEvent::InterfaceDown), &OspfIsm::interface_down);

        self.actions.insert((IsmState::DR, IsmEvent::NeighborChange), &OspfIsm::neighbor_change);
        self.actions.insert((IsmState::DR, IsmEvent::LoopInd), &OspfIsm::loop_ind);
        self.actions.insert((IsmState::DR, IsmEvent::InterfaceDown), &OspfIsm::interface_down);
    }

    pub fn call(&self, oi: &mut OspfInterface, state: IsmState, event: IsmEvent) -> IsmState {
        if let Some(action) = self.actions.get(&(state, event)) {
            debug!("ISM[-]: {} ({}): Action performed", state, event);
            action(oi)
        } else {
            debug!("ISM[-]: {} ({}): Ignored", state, event);
            oi.state
        }
    }

    /// Loopback action.
    pub fn loop_ind(oi: &mut OspfInterface) -> IsmState {
        IsmState::Loopback
    }

    pub fn unloop_ind(oi: &mut OspfInterface) -> IsmState {
        IsmState::Loopback
    }

    /// InterfaceUp action.
    pub fn interface_up(oi: &mut OspfInterface) -> IsmState {
        let next_state = oi.state;

        next_state
    }

    /// InterfaceDown action.
    pub fn interface_down(oi: &mut OspfInterface) -> IsmState {
        let next_state = oi.state;

        next_state
    }

    /// BackupSeen action.
    pub fn backup_seen(oi: &mut OspfInterface) -> IsmState {
        let next_state = oi.state;

        next_state
    }

    /// WaitTimer action.
    pub fn wait_timer(oi: &mut OspfInterface) -> IsmState {
        let next_state = oi.state;

        next_state
    }

    /// NeighborChange action.
    pub fn neighbor_change(oi: &mut OspfInterface) -> IsmState {
        let next_state = oi.state;

        // Perform DR Election.

        // => DR, DROther, Backup
        next_state
    }

    /// Handle ISM Event.
    pub fn handle_event(&self, oi: &mut OspfInterface, event: IsmEvent) {
        let state = oi.state;
        let next_state = self.call(oi, state, event);

        if next_state != state {
            self.change_state(oi, next_state);
        }
    }

    /// Change ISM State.
    pub fn change_state(&self, oi: &mut OspfInterface, state: IsmState) {
        // Logging.
        // Update stats.

        // RFC2328 s9.4(6).

        // Originate Network-LSA.

        // ABS status update.
    }
}
