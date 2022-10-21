//! Client events.
use std::fmt;
use std::io;
use std::sync::Arc;

use nakamoto_common::bitcoin::network::constants::ServiceFlags;
use nakamoto_common::bitcoin::{Transaction, Txid};
use nakamoto_common::block::{BlockHash, BlockHeader, Height};
use nakamoto_net::DisconnectReason;
use nakamoto_p2p::fsm;
use nakamoto_p2p::fsm::fees::FeeEstimate;
use nakamoto_p2p::fsm::{ConnDirection, PeerId};

use crate::spv::TxStatus;

/// Event emitted by the client during the "loading" phase.
#[derive(Clone, Debug)]
pub enum Loading {
    /// A block header was loaded from the store.
    /// This event only fires during startup.
    BlockHeaderLoaded {
        /// Height of loaded block.
        height: Height,
    },
    /// A filter header was loaded from the store.
    /// This event only fires during startup.
    FilterHeaderLoaded {
        /// Height of loaded filter header.
        height: Height,
    },
    /// A filter header was verified.
    /// This event only fires during startup.
    FilterHeaderVerified {
        /// Height of verified filter header.
        height: Height,
    },
}

impl fmt::Display for Loading {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockHeaderLoaded { height } => {
                write!(fmt, "block header #{} loaded", height)
            }
            Self::FilterHeaderLoaded { height } => {
                write!(fmt, "filter header #{} loaded", height)
            }
            Self::FilterHeaderVerified { height } => {
                write!(fmt, "filter header #{} verified", height)
            }
        }
    }
}

/// Event emitted by the client, after the "loading" phase is over.
#[derive(Debug, Clone)]
pub enum Event {
    /// Ready to process peer events and start receiving commands.
    /// Note that this isn't necessarily the first event emitted.
    Ready {
        /// The tip of the block header chain.
        tip: Height,
        /// The tip of the filter header chain.
        filter_tip: Height,
    },
    /// Peer connected. This is fired when the physical TCP/IP connection
    /// is established. Use [`Event::PeerNegotiated`] to know when the P2P handshake
    /// has completed.
    PeerConnected {
        /// Peer address.
        addr: PeerId,
        /// Connection link.
        link: ConnDirection,
    },
    /// Peer disconnected after successful connection.
    PeerDisconnected {
        /// Peer address.
        addr: PeerId,
        /// Reason for disconnection.
        reason: DisconnectReason<fsm::DisconnectReason>,
    },
    /// Connection was never established and timed out or failed.
    PeerConnectionFailed {
        /// Peer address.
        addr: PeerId,
        /// Connection error.
        error: Arc<io::Error>,
    },
    /// Peer handshake completed. The peer connection is fully functional from this point.
    PeerNegotiated {
        /// Peer address.
        addr: PeerId,
        /// Connection link.
        link: ConnDirection,
        /// Peer services.
        services: ServiceFlags,
        /// Peer height.
        height: Height,
        /// Peer user agent.
        user_agent: String,
        /// Negotiated protocol version.
        version: u32,
    },
    /// The best known height amongst connected peers has been updated.
    /// Note that there is no guarantee that this height really exists;
    /// peers don't have to follow the protocol and could send a bogus
    /// height.
    PeerHeightUpdated {
        /// Best block height known.
        height: Height,
    },
    /// A block was added to the main chain.
    BlockConnected {
        /// Block header.
        header: BlockHeader,
        /// Block hash.
        hash: BlockHash,
        /// Height of the block.
        height: Height,
    },
    /// One of the blocks of the main chain was reverted, due to a re-org.
    /// These events will fire from the latest block starting from the tip, to the earliest.
    /// Mark all transactions belonging to this block as *unconfirmed*.
    BlockDisconnected {
        /// Header of the block.
        header: BlockHeader,
        /// Block hash.
        hash: BlockHash,
        /// Height of the block when it was part of the main chain.
        height: Height,
    },
    /// A block has matched one of the filters and is ready to be processed.
    /// This event usually precedes [`Event::TxStatusChanged`] events.
    BlockMatched {
        /// Hash of the matching block.
        hash: BlockHash,
        /// Block header.
        header: BlockHeader,
        /// Block height.
        height: Height,
        /// Transactions in this block.
        transactions: Vec<Transaction>,
    },
    /// Transaction fee rate estimated for a block.
    FeeEstimated {
        /// Block hash of the estimate.
        block: BlockHash,
        /// Block height of the estimate.
        height: Height,
        /// Fee estimate.
        fees: FeeEstimate,
    },
    /// A filter was processed. If it matched any of the scripts in the watchlist,
    /// the corresponding block was scheduled for download, and a [`Event::BlockMatched`]
    /// event will eventually be fired.
    FilterProcessed {
        /// Corresponding block hash.
        block: BlockHash,
        /// Filter height (same as block).
        height: Height,
        /// Whether or not this filter matched any of the watched scripts.
        matched: bool,
        /// Whether or not this filter is valid.
        valid: bool,
    },
    /// The status of a transaction has changed.
    TxStatusChanged {
        /// The Transaction ID.
        txid: Txid,
        /// The new transaction status.
        status: TxStatus,
    },
    /// Compact filters have been synced and processed up to this point and matching blocks have
    /// been fetched.
    ///
    /// If filters have been processed up to the last block in the client's header chain, `height`
    /// and `tip` will be equal.
    Synced {
        /// Height up to which we are synced.
        height: Height,
        /// Tip of our block header chain.
        tip: Height,
    },
}

impl fmt::Display for Event {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready { .. } => {
                write!(fmt, "ready to process events and commands")
            }
            Self::BlockConnected { hash, height, .. } => {
                write!(fmt, "block {} connected at height {}", hash, height)
            }
            Self::BlockDisconnected { hash, height, .. } => {
                write!(fmt, "block {} disconnected at height {}", hash, height)
            }
            Self::BlockMatched { hash, height, .. } => {
                write!(
                    fmt,
                    "block {} ready to be processed at height {}",
                    hash, height
                )
            }
            Self::FeeEstimated { fees, height, .. } => {
                write!(
                    fmt,
                    "transaction median fee rate for block #{} is {} sat/vB",
                    height, fees.median,
                )
            }
            Self::FilterProcessed {
                height, matched, ..
            } => {
                write!(
                    fmt,
                    "filter processed at height {} (match = {})",
                    height, matched
                )
            }
            Self::TxStatusChanged { txid, status } => {
                write!(fmt, "transaction {} status changed: {}", txid, status)
            }
            Self::Synced { height, .. } => write!(fmt, "filters synced up to height {}", height),
            Self::PeerConnected { addr, link } => {
                write!(fmt, "peer {} connected ({:?})", &addr, link)
            }
            Self::PeerConnectionFailed { addr, error } => {
                write!(
                    fmt,
                    "peer connection attempt to {} failed with {}",
                    &addr, error
                )
            }
            Self::PeerHeightUpdated { height } => {
                write!(fmt, "peer height updated to {}", height)
            }
            Self::PeerDisconnected { addr, reason } => {
                write!(fmt, "disconnected from {} ({})", &addr, reason)
            }
            Self::PeerNegotiated {
                addr,
                height,
                services,
                ..
            } => write!(
                fmt,
                "peer {} negotiated with services {} and height {}..",
                addr, services, height
            ),
        }
    }
}
