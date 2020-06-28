//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF constatns and enums.
//

use std::fmt;

/// LSA type (supposed to be 0 origin).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OspfLsaType {

    /// Type 0: Unknown (Reserved)
    Unknown,

    /// Type 1: Router-LSA.
    RouterLsa,

    /// Type 2: Network-LSA.
    NetworkLsa,

    /// Type 3: Summary-LSA.
    SummaryLsa,

    /// Type 4: Summary-LSA (ASBR).
    AsbrSummaryLsa,

    /// Type 5: AS-External-LSA.
    AsExternalLsa,

    /// Type 6: Group-Membership-LSA (RFC1584, RFC5110, not supported).
    GroupMembershipLsa,

    /// Type 7: NSSA AS-External-LSA.
    NssaAsExternalLsa,

    /// Type 8:External-Attributes-LSA (???)
    ExternalAttributesLsa,

    /// Type 9: Link-Scoped Opaque LSA.
    LinkScopedOpaqueLsa,

    /// Type 10: Area-Scoped Opaque LSA.
    AreaScopedOpaqueLsa,

    /// Type 11: AS-Scoped Opaque LSA.
    AsScopedOpaqueLsa,
}

impl OspfLsaType {

    pub fn to_string(&self) -> &str {
        match *self {
            OspfLsaType::Unknown => "Unknown",
            OspfLsaType::RouterLsa => "Router-LSA",
            OspfLsaType::NetworkLsa => "Network-LSA",
            OspfLsaType::SummaryLsa => "Summary-LSA",
            OspfLsaType::AsbrSummaryLsa => "ASBR-Summary-LSA",
            OspfLsaType::AsExternalLsa => "AS-External-LSA",
            OspfLsaType::GroupMembershipLsa => "Group-Membreship-LSA",
            OspfLsaType::NssaAsExternalLsa => "NSSA-AS-External-LSA",
            OspfLsaType::ExternalAttributesLsa => "External-Attributes-LSA",
            OspfLsaType::LinkScopedOpaqueLsa => "Link-Scoped-Opaque-LSA",
            OspfLsaType::AreaScopedOpaqueLsa => "Area-Scoped-Opaque-LSA",
            OspfLsaType::AsScopedOpaqueLsa => "AS-Scoped-Opaque-LSA",
        }
    }
}

impl fmt::Display for OspfLsaType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
}

impl fmt::Debug for OspfLsaType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_string())
    }
}

const MIN_LSA: u8 = 1;
const MAX_LSA: u8 = 12;

/// Flooding Scope.
pub enum OspfFloodingScope {

    /// Link Scope.
    Link,

    /// Area Scope.
    Area,

    /// AS Scope.
    As,
}

/// OSPF Auth Type.
pub enum OspfAuth {

    /// 0: No Authentication (default)
    NoAuthentication,

    /// 1: Simple Password Authentication
    SimplePassword,

    /// 2: Cryptographic Authentication (RFC2328, RFC5709)
    Cryptographic,

    /// 3: Cryptographic Authentication with Extended Sequence Numbers (RFC7474)
    CryptographicExtended
}
