//! These are utility structs to perform filtering (etc) on [Leases].
//!
//! * LeasesFilter: First version of simple filtering
//! * LeaseFilterBuilder: More robust filtering
#[doc(inline)]
use crate::leases::{Lease, Leases};
#[doc(inline)]
use crate::leases::{LeasesMethods};


pub struct LeasesFilter {}

/// Filter leases matching criteria from a Leases
impl LeasesFilter {
    /// Returns a copy of all leases for the client with `mac`. The `mac` pattern can be a whole
    /// mac address, or the first n characters of a mac. The filter uses .starts_with to compare.
    pub fn by_mac_all(leases: &Leases, mac: &str) -> Vec<Lease> {
        let mut new_leases = Vec::new();

        for ndx in 0..leases.count() {
            if let Some(hw) = leases[ndx].hardware.as_ref() {
                if hw.mac.starts_with(&mac) {
                    new_leases.push(leases[ndx].clone());
                }
            }
        }

        new_leases
    }

    /// Returns a copy of all currently-active leases for the client
    pub fn by_mac_active(leases: &Leases, mac: &str) -> Vec<Lease> {
        let leases = LeasesFilter::by_mac_all(&leases, &mac);
        let mut active : Vec<Lease> = Vec::new();

        for lease in leases.iter() {
            if lease.is_active() {
                active.push(lease.clone());
            }
        }

        active
    }
}

/// Builder helper for constructing [Leases] filtering of [Lease] entries.
///
/// # Overview
///
/// The [Leases] instance will contain all the [Lease] items found in the "dhcpd.leases" file.
/// To filter the list with a set of criteria, add "on_" methods to narrow the scope. These should
/// be supplied from broad to narrow scope. Each "on_" method added will filter down the list
/// of matching leases such that the final call to [.collect()](LeaseFilterBuilder::collect) will
/// return a [Leases] instance with matching lease entries.
///
/// # Examples
///
/// ## Filter for active items on mac
///
/// The following will return a [Leases] instance with one item (as shows in the "simple" example,
/// using the "tests/data/dhcpd-linux.leases" file).
///
/// * `LeaseFilterBuilder::new(&leases)` constructs an instance of the builder
/// * `.on_mac` adds a filter for items matching the MAC address
/// * `.on_active` further filters for only those that are currently active
/// * `.collect` constructs an instance of [Leases] with a clone of the matching [Lease] items
///
///```rust
/// use dhcpd_parser::util::{LeaseFilterBuilder, LeasesFilter};
///
/// let mut builder = LeaseFilterBuilder::new(&leases);
/// let filtered = builder.on_mac("00:ad:d4:39:0d:04")
///    .on_active()
///    .collect();
/// ```
///
/// ## Filter on IP
///
/// This shows how we can review the history of an IP subnet to see which clients have been offered/assigned them.
/// Because we supplied a partial IP, this will match all entries with IP starting with "192.168.4". (Naturally,
/// for most DHCP servers, this would be _all_ the IPs.)
///
/// ```rust
/// use dhcpd_parser::util::{LeaseFilterBuilder, LeasesFilter};
///
/// let mut builder = LeaseFilterBuilder::new(&leases);
/// let filtered = builder.on_ip("192.168.4")
///     .collect();
/// ```
pub struct LeaseFilterBuilder {
    /// Contains a clone of the [Lease] items
    leases: Vec<Lease>,
    /// A vector of indexes into `leases` that is reduced for each "on_" method that is called. The final call to
    /// [LeaseFilterBuilder::collect] will return a [Leases] instance with a clone of each
    /// matching lease item.
    match_indexes: Vec<usize>,
}

impl LeaseFilterBuilder {
    /// Create a new instance with a clone of the [Lease] items.
    pub fn new(leases: &Leases) -> LeaseFilterBuilder {
        let mut leases_vec : Vec<Lease> = Vec::new();
        for ndx in 0..leases.count() {
            leases_vec.push(leases[ndx].clone())
        }

        LeaseFilterBuilder {leases: leases_vec, match_indexes: (0..leases.count()).collect()}
    }

    /// Add filtering on an IP address. Keep in mind that the IP can - and will - be offered
    /// to different clients unless a static entry assigns it to a specific MAC/client. Like
    /// the [on_mac](Self::on_mac) filter, this IP pattern can be a partial string with the first n characters.
    pub fn on_ip(&mut self, ip: &str) -> &mut Self {
        let mut keep_ndx : Vec<usize> = Vec::new();

        for ndx in self.match_indexes.iter() {
            let lease = &self.leases[*ndx];

            if lease.ip.starts_with(&ip) {
                keep_ndx.push(ndx.clone());
            }

        }

        self.match_indexes.retain(|&i| keep_ndx.contains(&i));

        self
    }

    /// Add filtering on a MAC address pattern. The pattern can be either the whole MAC
    /// address, or the first n characters - this allows filtering on all of the MACs for a
    /// particular vendor OUI/MA-L address block.
    pub fn on_mac(&mut self, mac: &str) -> &mut Self {
        let mut keep_ndx : Vec<usize> = Vec::new();

        for ndx in self.match_indexes.iter() {
            let lease = &self.leases[*ndx];

            if lease.client().starts_with(&mac) {
                keep_ndx.push(ndx.clone());
            }
        }

        self.match_indexes.retain(|&i| keep_ndx.contains(&i));

        self
    }

    /// Add filtering on [is_active](Lease::is_active). This will inspect both the [abandoned](Lease::abandoned)
    /// and the [binding_state](Lease::binding_state) field, if specified (Linux-only).
    pub fn on_active(&mut self) -> &mut Self {
        let mut keep_ndx : Vec<usize> = Vec::new();

        for ndx in self.match_indexes.iter() {
            let lease = &self.leases[*ndx];

            if lease.is_active() {
                keep_ndx.push(ndx.clone());
            }
        }

        self.match_indexes.retain(|&i| keep_ndx.contains(&i));

        self
    }

    /// Collect the filtered leases as a [Leases] object. Each [Lease] is cloned into
    /// the new instance. If there are no matching lease items after the filtering, the count
    /// from [Leases::count] will be 0. (This uses the private _match_indexes_ vector.)
    pub fn collect(&mut self) -> Leases {
        let mut leases = Leases::new();

        for ndx in self.match_indexes.iter() {
            leases.push(self.leases[*ndx].clone())
        }

        leases
    }
}